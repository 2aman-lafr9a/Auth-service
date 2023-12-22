use std::env;
use bcrypt::{DEFAULT_COST, hash};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, encode, EncodingKey, Header};
use redis::{Commands, RedisResult};
use tonic::{Request, Response, Status, transport::Server};
use authentication::{authentication_server::{Authentication, AuthenticationServer}, SignInRequest, SignInResponse, SignUpRequest, SignUpResponse};
use database::redis_connection::redis_connect;

mod database;
mod models;
mod schema;
mod helpers;


pub mod authentication {
    tonic::include_proto!("authentication");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let address = "127.0.0.1:8080".parse().unwrap();
    let authentication_service = AuthenticationService::default();

    Server::builder()
        .add_service(AuthenticationServer::new(authentication_service))
        .serve(address)
        .await?;

    println!("Server listening on {}", address);

    Ok(())
}


#[derive(Debug, Default)]
pub struct AuthenticationService {}


#[tonic::async_trait]
impl Authentication for AuthenticationService {
    async fn sign_up(&self, request: Request<SignUpRequest>) -> Result<Response<SignUpResponse>, Status> {
        // get the user from the request
        let user = request.into_inner();

        // get the username, password, and role from the request
        let username = user.username;
        let password = user.password;
        let role = user.role;

        // validate the user role
        if !helpers::is_valid_user_role(&role) {
            return Err(Status::invalid_argument("Invalid user role"));
        }

        // validate the username
        if !helpers::is_valid_username(&username) {
            return Err(Status::invalid_argument("Invalid username"));
        }

        // validate the password
        if !helpers::is_valid_password(&password) {
            return Err(Status::invalid_argument("Invalid password"));
        }

        // connect to redis
        let mut redis_connection = redis_connect();

        // check if the user already exists in redis
        let redis_key = format!("user:{}", username);
        let redis_value: RedisResult<String> = redis_connection.get(&redis_key);

        // if the user already exists, return an error (username must be unique)
        match redis_value {
            Ok(_) => {
                return Err(Status::already_exists("User already exists"));
            }
            Err(_) => {}
        }
        // connect to mysql
        let mut mysql_connection = database::mysql_connection::mysql_connect();

        // check if the user exists in the database already
        // if the user exists, return an error
        let result = models::user::User::find_user_by_username(&mut mysql_connection, &username);

        match result {
            Ok(optional_user) => {
                match optional_user {
                    None => {}
                    Some(_) => {
                        return Err(Status::already_exists("User already exists"));
                    }
                }
            }
            Err(_) => {}
        }

        // hash the password
        let hashed_password = match hash(&password, DEFAULT_COST) {
            Ok(hashed) => hashed,
            Err(_) => return Err(Status::internal("Failed to hash password")),
        };

        // save the user to redis
        let redis_value = format!("{}:{}", hashed_password, role);

        // save the user to the database
        let result: RedisResult<()> = redis_connection.set(redis_key, redis_value);

        match result {
            Ok(_) => {}
            Err(_) => {
                print!("Failed to create user");
                return Err(Status::internal("Failed to create user"));
            }
        }
        mysql_connection = database::mysql_connection::mysql_connect();


        // save the user to the database
        let result = models::user::User::create_user(&mut mysql_connection, &username, &hashed_password, &role);

        match result {
            Ok(_) => {}
            Err(error) => {
                print!("Failed to create user {:?}", error);
                return Err(Status::internal("Failed to create user"));
            }
        }


        // return a success response
        let response = SignUpResponse {
            success: true,
            message: "User created successfully".into(),
        };

        return Ok(Response::new(response));
    }

    async fn sign_in(&self, request: Request<SignInRequest>) -> Result<Response<SignInResponse>, Status> {
        // get the user from the request
        let user = request.into_inner();

        // get the username and password from the request
        let username = user.username;
        let password = user.password;

        // connect to redis
        let mut redis_connection = redis_connect();

        // check if the user exists in redis
        let redis_key = format!("user:{}", username);
        let redis_value: RedisResult<String> = redis_connection.get(&redis_key);
        let role;

        // Define the secret key
        dotenv().ok();
        let secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

        match redis_value {
            Ok(user) => {
                // the value in redis is in the format "hashed_password:role", so split it to get the hashed password
                let redis_hashed_password = user.split(":").next().unwrap();
                role = user.split(":").last().unwrap().to_string();

                // compare the hashed password from redis with the hashed version of the password from the request
                if bcrypt::verify(&password, redis_hashed_password).is_err() {
                    return Err(Status::unauthenticated("Invalid password"));
                }
                Self::return_jwt(username, role, &secret_key)
            }
            Err(_) => {
                // connect to mysql
                let mut mysql_connection = database::mysql_connection::mysql_connect();
                // check if the user exists in the database
                let result = models::user::User::find_user_by_username(&mut mysql_connection, &username);

                match result {
                    Ok(user) => {
                        return match user {
                            Some(user) => {
                                role = user.role;

                                // compare the hashed password from the database with the hashed version of the password from the request
                                if bcrypt::verify(&password, &user.password_hash).is_err() {
                                    return Err(Status::unauthenticated("Invalid credentials"));
                                }

                                Self::return_jwt(username, role, &secret_key)
                            }
                            None => Err(Status::unauthenticated("Invalid credentials"))
                        };
                    }
                    Err(_) => Err(Status::internal("Can't get a user"))
                }
            }
        }
    }
}

impl AuthenticationService {
    fn return_jwt(username: String, role: String, secret_key: &String) -> Result<Response<SignInResponse>, Status> {
        // Create the claims
        let claims = helpers::Claims {
            username,
            role,
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret_key.as_ref()))
            .expect("Failed to encode claims into JWT");


        // return a success response
        let response = SignInResponse {
            success: true,
            message: "User signed in successfully".into(),
            jwt: token,
        };
        Ok(Response::new(response))
    }
}

// todo imp singleton pattern in connection
//  todo right test


