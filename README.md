Chatty Log Parser
----------------

Parses chatty logs and saves them into a database. Specifically intended for logs that include
interruptions from crashes and/or log formats which do not include a full timestamp before
each Message.

### Set up Database

1. Copy .env-sample to .env and enter your database's connection details.

2. Run migrations:
    ```
    diesel migration run
    ```