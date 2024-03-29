use std::env;
use std::time::{Duration, SystemTime};
use bcrypt::{DEFAULT_COST, hash};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use redis::{Commands, RedisResult};
use tonic::{Request, Response, Status, transport::Server};
use authentication::{authentication_server::{Authentication, AuthenticationServer}, SignInRequest, SignInResponse, SignUpRequest, SignUpResponse};
use database::redis_connection::redis_connect;
use crate::authentication::{TokenValidationRequest, TokenValidationResponse};
use crate::helpers::Claims;


mod database;
mod models;
mod schema;
mod helpers;


pub mod authentication {
    tonic::include_proto!("authentication");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| String::from("5000"));
    let address = format!("0.0.0.0:{}", port).parse().unwrap();
    let authentication_service = AuthenticationService::default();

    println!("Server listening on {}", address);
    Server::builder()
        .add_service(AuthenticationServer::new(authentication_service))
        .serve(address)
        .await?;


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
            Err(error) => {
                print!("Failed to create user{error}");
                return Err(Status::internal("Failed to create user"));
            }
        }
        mysql_connection = database::mysql_connection::mysql_connect();


        // save the user to the database
        let result = models::user::User::create_user(&mut mysql_connection, &username, &hashed_password, &role);

        match result {
            Ok(_) => {}
            Err(error) => {
                println!("Failed to create user {error}");
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
                    Err(error) => {
                        println!("Failed to get user {error}");
                        Err(Status::internal("Can't get a user "))
                    }
                }
            }
        }
    }
    async fn validate_token(&self, request: Request<TokenValidationRequest>) -> Result<Response<TokenValidationResponse>, Status> {
        // get the user from the request
        let user = request.into_inner();

        // get the username and password from the request
        let token = user.jwt;

        // Define the secret key
        dotenv().ok();
        let secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

        // `token` is a struct with 2 fields: `header` and `claims` where `claims` is your own struct.
        let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(secret_key.as_ref()), &Validation::default());

        match token_data {
            Ok(data) => {
                // return a success response
                let response = TokenValidationResponse {
                    success: true,
                    message: "JWT is valid".into(),
                    username: data.claims.username,
                    role: data.claims.role,
                    expiration: data.claims.exp.to_string(),
                };
                Ok(Response::new(response))
            }
            Err(error) => {
                println!("Failed to validate token {error}");
                return Err(Status::unauthenticated("Invalid token"));
            }
        }
    }
}

impl AuthenticationService {
    fn return_jwt(username: String, role: String, secret_key: &String) -> Result<Response<SignInResponse>, Status> {
        let expiration = env::var("JWT_EXPIRATION")
            .expect("JWT_EXPIRATION must be set")
            .parse::<u64>()
            .expect("JWT_EXPIRATION must be an integer");
        // Get the current time
        let current_time = SystemTime::now();
        // Add the expiration time to the current time
        let duration_to_add = Duration::from_secs(expiration);
        // Convert the current time to a usize
        let exp = (current_time + duration_to_add)
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get UNIX_EPOCH")
            .as_secs() as usize;

        // Create the claims
        let claims = Claims {
            username,
            role,
            exp,
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

