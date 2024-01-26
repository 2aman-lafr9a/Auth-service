1. **Auth service:**
    
    

This microserver is responsible for JWT authentication, and role-based authorization, it is written with a low level language and a fast language **RUST** (Rust is a multi-paradigm, general-purpose programming language that emphasizes performance, type safety, and concurrency. It enforces memory safety, meaning that all references point to valid memory, without requiring the use of automated memory management techniques, such as garbage collector) 

[https://en.wikipedia.org/wiki/Rust_(programming_language)](https://en.wikipedia.org/wiki/Rust_(programming_language))

It uses a MySql database ( Relational database ) as  a primary database to store its data, and redis to cache the user data so the access to db will be fast, and it provides a GRPC interface to deal with it, it’s dumps and all that it knows is AUTH only, the methods it provides are 

```protobuf
rpc SignUp(SignUpRequest) returns (SignUpResponse);
rpc SignIn(SignInRequest) returns (SignInResponse);
rpc validateToken(TokenValidationRequest) returns (TokenValidationResponse);
```

what makes this microservice essential and special, it’s stateless, reusable, and central, but it’s too crucial because it’s a point of failure, and it’s used by the API gateway to validate authorizations and make sure that all routes are protected, and it hides the details of auth from any other microservers so devs can focus only on the business logic of their microservices.
