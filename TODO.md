* [x] move app code to separate crate
* [x] better logging than println!

* [ ] improve data quality
    * [x] capture job name
    * [x] capture other details
    * [x] make unique traceid/spanid for reruns of a pipeline

* [x] configure direct export to honeycomb, instead of going through the collector
* [ ] create a container for running the hook somewhere/anywhere
* [ ] Fully implement the entire schema in structs; remove serde_json::Value and dependency
* [ ] handle service configuration
    * [x] key or no secret
    * [ ] tracer target config

* [ ] error handling
    * [ ] replace &str returns with thiserror error for the request
    * [ ] parse_signature_header
    * [x] all the todo!s
