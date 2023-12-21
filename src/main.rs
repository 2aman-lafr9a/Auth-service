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
    let address = "[::1]:8080".parse().unwrap();
    let authentication_service = AuthenticationService::default();

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
        let username = helpers::sanitize_input(user.username);
        let password = helpers::sanitize_input(user.password);
        let role = helpers::sanitize_input(user.role);

        // validate the user role
        if !helpers::validate_user_role(&role) {
            return Err(Status::invalid_argument("Invalid user role"));
        }

        // validate the username
        if !helpers::validate_user_name(&username) {
            return Err(Status::invalid_argument("Invalid username"));
        }

        // validate the password
        if !helpers::validate_password(&password) {
            return Err(Status::invalid_argument("Invalid password"));
        }

        // connect to redis
        let mut redis_connection = redis_connect();

        // check if the user already exists in redis
        let redis_key = format!("user:{}", username);
        let redis_value: RedisResult<String> = redis_connection.get(&redis_key);

        // if the user already exists, return an error (username must be unique)
        if redis_value.is_ok() {
            return Err(Status::already_exists("User already exists"));
        }

        // connect to mysql
        let mut mysql_connection = database::mysql_connection::mysql_connect();

        // check if the user exists in the database already
        // if the user exists, return an error
        let result = models::user::User::find_user_by_username(&mut mysql_connection, &username);
        if result.is_ok() {
            return Err(Status::already_exists("User already exists"));
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
        if result.is_err() {
            return Err(Status::internal("Failed to create user"));
        }

        // save the user to the database
        let result = models::user::User::create_user(&mut mysql_connection, &username, &hashed_password, &role);
        if result.is_err() {
            return Err(Status::internal("Failed to create user"));
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
        let username = helpers::sanitize_input(user.username);
        let password = helpers::sanitize_input(user.password);

        // connect to redis
        let mut redis_connection = redis_connect();

        // check if the user exists in redis
        let redis_key = format!("user:{}", username);
        let redis_value: RedisResult<String> = redis_connection.get(&redis_key);
        let role;

        // if the user doesn't exist in redis, check in the database
        if redis_value.is_err() {
            // connect to mysql
            let mut mysql_connection = database::mysql_connection::mysql_connect();

            // check if the user exists in the database
            let result = models::user::User::find_user_by_username(&mut mysql_connection, &username);
            if result.is_err() {
                return Err(Status::not_found("User not found"));
            }

            // get the user from the database
            let db_user = result.unwrap().unwrap();

            // get the role from the database
            role = db_user.role;

            // compare the hashed password from the database with the hashed version of the password from the request
            if bcrypt::verify(&password, &db_user.password_hash).is_err() {
                return Err(Status::unauthenticated("Invalid password"));
            }
        } else {
            // get the user from redis
            let redis_user = redis_value.unwrap();

            // the value in redis is in the format "hashed_password:role", so split it to get the hashed password
            let redis_hashed_password = redis_user.split(":").next().unwrap();
            role = redis_user.split(":").last().unwrap().to_string();

            // compare the hashed password from redis with the hashed version of the password from the request
            if bcrypt::verify(&password, redis_hashed_password).is_err() {
                return Err(Status::unauthenticated("Invalid password"));
            }
        }
        // Define the secret key
        dotenv().ok();
        let secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");

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
        return Ok(Response::new(response));
    }
}

// todo imp singleton pattern in connection
//  todo right test

