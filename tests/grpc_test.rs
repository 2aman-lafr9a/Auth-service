pub mod authentication {
    tonic::include_proto!("authentication");
}


#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod tests {
        use std::env;
        use fake::faker::internet::en::{Password, Username};
        use crate::authentication::{SignUpRequest};
        use crate::authentication::authentication_client::AuthenticationClient;

        #[tokio::test]
        async fn sign_up() {
            let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
            let address = format!("localhost:{}", port).parse().unwrap();
            // creating a channel ie connection to server
            let channel = tonic::transport::Channel::from_static(address)
                .connect()
                .await;
            match &channel {
                Ok(_) => println!("Channel created"),
                Err(err) => {
                    println!("ERROR={:?}", err);
                    assert!(false);
                }
            }
            // creating gRPC client from channel
            let mut client = AuthenticationClient::new(channel);

            // ? valid user sign up
            // creating a new Request
            let request = tonic::Request::new(
                SignUpRequest {
                    username: Username().into(),
                    password: Password(8..20).into(),
                    role: String::from("team_manager"),
                },
            );
            // sending request and waiting for response
            let response = client.sign_in(request).await;
            match response {
                Ok(response) => {
                    println!("RESPONSE={:?}", response);
                    assert!(response.get_ref().success);
                }
                Err(err) => {
                    println!("ERROR={:?}", err);
                    assert!(false);
                }
            }
        }
    }
}