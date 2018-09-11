# MyPaste project

This project is intended to simulate an entire infrastructure for a service such as the pastebin service.

It shall run some Docker instances for a cache (e.g. Redis) and an object store (e.g. MongoDB). Perhaps, it would be interesting to also have some search capabilities with, e.g., ElasticSearch.

The webserver that receives and responds requests will be written in the Rust Programming Language and should not make assumptions about the technologies in use in the Docker instances.
