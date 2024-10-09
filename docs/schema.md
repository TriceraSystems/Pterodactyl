
# Request

All requests are POST requests to the same route, intention is specified within payload.

```js
{
    "method": string,   // HTTP request methods
    "cache": boolean,   // Do you want response from cache (cost extra for non-cache)
    "process": string,  // The process ID you want to call
    "payload": {}       // The data passed to process
}
```

# Response

All responses follow the same schema however internal standardisation is needed for the data.

```js
{
    "code": number,     // HTTP status code
    "message": string,  // Small response summary
    "data": {},         // Proccessed Data
    "errors": [],       // Any errors
    "timestamp": string,// Timestamp of process response (when it was created)
    "cache": boolean,   // Is the response from cache
    "cost": number,     // Processing cost
    "hash": string      // SHA256 Hash of response excluding self
}
```
