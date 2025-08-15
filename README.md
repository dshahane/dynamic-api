# **Dynamic CRUD API in Rust**

This project is a Rust-based web service that provides a dynamic CRUD (Create, Read, Update, Delete) API. The data model for the API is not hard-coded but is instead defined at runtime by a JSON Schema uploaded by the user.

The service is built on the **Actix Web** framework and uses several key libraries to handle schema validation, data management, and API documentation. It features a built-in Swagger UI for easy exploration of the available endpoints.

## **✨ Features**

* **Dynamic Data Models:** Define new data types at runtime by uploading a JSON Schema.
* **Real-time Validation:** All data sent to the API is validated against the active schema for its corresponding data model.
* **In-Memory Storage:** The service uses an in-memory hash map for fast and simple data persistence.
* **Automated OpenAPI Documentation:** The API endpoints are documented using utoipa procedural macros, which automatically generates an OpenAPI specification.
* **Built-in Swagger UI:** Access an interactive API documentation page directly from your browser.

## **🚀 How to Run**

### **Prerequisites**

* Rust and Cargo installed on your system. You can install them with [rustup](https://rustup.rs/).

### **Steps**

1. Clone this repository to your local machine.
2. Navigate to the project directory.
3. Build and run the project with Cargo:  
   cargo run

The service will start on port 7777\. You can then access the following URLs:

* **Service Root:** http://127.0.0.1:7777
* **Swagger UI:** http://127.0.0.1:7777/swagger-ui/

## **📚 API Endpoints**

The API consists of a set of generic endpoints that operate on any data model defined by a user-uploaded schema.

| Method | Path | Description |
| :---- | :---- | :---- |
| POST | /api/schema | Uploads a new JSON Schema to define a data model. |
| POST | /api/{model\_name} | Creates a new item for the specified data model. |
| GET | /api/{model\_name}/{id} | Retrieves a specific item by its unique ID. |
| PUT | /api/{model\_name}/{id} | Updates an existing item. |
| DELETE | /api/{model\_name}/{id} | Deletes a specific item. |

## **💡 Example Usage**

Here is a full example of the API flow using curl.

1. **Upload a Schema for a Task Model**  
   curl \-X POST http://127.0.0.1:7777/api/schema \\  
   \-H "Content-Type: application/json" \\  
   \-d '{  
   "name": "Task",  
   "schema": {  
   "type": "object",  
   "properties": {  
   "title": { "type": "string" },  
   "completed": { "type": "boolean" }  
   },  
   "required": \["title", "completed"\]  
   }  
   }'

2. **Create a New Task**  
   curl \-X POST http://127.0.0.1:7777/api/Task \\  
   \-H "Content-Type: application/json" \\  
   \-d '{"title": "Write README", "completed": false}'

   *(Save the id from the response for the next steps.)*
3. **Retrieve the Task**  
   curl \-X GET http://127.0.0.1:7777/api/Task/{id}

4. **Update the Task**  
   curl \-X PUT http://127.0.0.1:7777/api/Task/{id} \\  
   \-H "Content-Type: application/json" \\  
   \-d '{"title": "Write README", "completed": true}'

5. **Delete the Task**  
   curl \-X DELETE http://127.0.0.1:7777/api/Task/{id}  
