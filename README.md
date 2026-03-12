# Scenario-Queue Manager

This is a task manager for the PISA Scenario-Queue project. It is responsible for managing the tasks that are created by the users and assigning them to the appropriate executors.

## Usage

### Configuration

Copy the `.env.example` file to `.env` and fill in the required environment variables.

### Running the Application

You can run the application using Docker Compose:

```bash
docker-compose up --build
```

It will start the following services:
- `postgres`: The PostgreSQL database for storing task and user data.
- `postgrest`: The PostgREST server for providing a RESTful API to the database.
- `manager`: The main application that manages the tasks.
- `swagger-ui`: The Swagger UI for API documentation and testing.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue if you have any ideas or suggestions.
