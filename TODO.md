* ☑️ move app code to separate crate
* ☑️ better logging than println!

* improve data quality 
    * ☑️ capture job name
    * ☑️ capture other details
    * make unique traceid/spanid for reruns of a pipeline

* configure direct export to honeycomb, instead of going through the collector
* create a container for running the hook somewhere/anywhere
* Fully implement the entire schema in structs; remove serde_json::Value and dependency
* handle service configuration
    * key or no secret
    * tracer target config

* error handling
    * replace &str returns with thiserror error for the request
    * parse_signature_header
    * all the todo!s
