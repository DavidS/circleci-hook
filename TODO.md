* [ ] write README explaining project and deployment options

* [ ] improve data quality
    * [x] capture job name
    * [x] capture other details
    * [x] make unique traceid/spanid for reruns of a pipeline
    * [ ] fetch step info from API and send as spans
    * [x] provide a way to identify the trace_id, span_id of the currently running step on CircleCI
        * [x] inject a TRACEPARENT into the circleci environment
            * [x] provide an endpoint to translate circleci environment into a TRACEPARENT value
* [ ] Fully implement the entire schema in structs; remove serde_json::Value and dependency
* [ ] handle service configuration
    * [x] key or no secret
    * [ ] tracer target config

* [x] move app code to separate crate
* [x] better logging than println!
* [x] configure direct export to honeycomb, instead of going through the collector
* [x] create a container for running the hook somewhere/anywhere
* [x] error handling
    * [x] replace &str returns with thiserror error for the request
    * [x] parse_signature_header
    * [x] all the todo!s
