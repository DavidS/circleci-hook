* move app code to separate crate
* improve data quality 
    * capture job name
    * make unique traceid/spanid for reruns of a pipeline
* better logging than println!
* configure direct export to honeycomb, instead of going through the collector
* create a container for running the hook somewhere/anywhere