# Archetype Rust Axon

An archetypal Rust project that uses AxonServer for Event Sourcing and CQRS.
This project is a sibling of [archetype-go-axon](https://github.com/dendrite2go/archetype-go-axon), but for the Rust programming language.
This project uses the [dendrite crate](https://crates.io/crates/dendrite) to connect to [AxonServer](https://axoniq.io/product-overview/axon-server).

# Core concepts

* [Command / Query Responsibility Segregation](http://codebetter.com/gregyoung/2010/02/16/cqrs-task-based-uis-event-sourcing-agh/) (CQRS)
* [Domain-driven Design](https://dddcommunity.org/learning-ddd/what_is_ddd/) (DDD)
* [Event Sourcing](https://axoniq.io/resources/event-sourcing)
* [Futures and async/await](https://rust-lang.github.io/async-book)
* [gRPC](https://grpc.io/)
* [Microservices](https://en.wikipedia.org/wiki/Microservices)
* [Service Mesh](https://buoyant.io/2017/04/25/whats-a-service-mesh-and-why-do-i-need-one/) (video: [talk by Jeroen Reijn](https://2019.jfall.nl/sessions/whats-a-service-mesh-and-why-do-i-need-one/))

# Stack

In alphabetic order:

* [Bash](https://www.gnu.org/software/bash/manual/bash.html): The shell, or command language interpreter, for the GNU operating system — _for building and deploying_
* [AxonServer](https://axoniq.io/product-overview/axon-server): A zero-configuration message router and event store for Axon ([docker image](https://hub.docker.com/r/axoniq/axonserver/)) — _Event Store_
* [Docker compose](https://docs.docker.com/compose/): A tool for defining and running multi-container Docker applications — _for spinning up development and test environments_
* [ElasticSearch](https://www.elastic.co/elasticsearch/) You know, for search ([docker image](https://hub.docker.com/_/elasticsearch)) — _for query models (though any tokio-compatible persistence engine will do)_
* [Envoy proxy](https://www.envoyproxy.io/): An open source edge and service proxy, designed for cloud-native applications ([docker image](https://hub.docker.com/u/envoyproxy/)) — _to decouple microservices_
* [React](https://reactjs.org/): A JavaScript library for building user interfaces — _for the front-end_
* [Rust](https://www.rust-lang.org): A language empowering everyone to build reliable and efficient software — _for the back-end_
* [Tonic](https://github.com/hyperium/tonic): A Rust implementation of [gRPC](https://grpc.io/) with first class support of async/await — _for the plumbing on the back-end_
