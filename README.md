# Archetype Rust Axon

An archetypal Rust project that uses AxonServer for Event Sourcing and CQRS.
This project uses the [dendrite crate](https://crates.io/crates/dendrite) ([rustic-dendrite git repo](https://github.com/dendrite2go/rustic-dendrite)) to connect to [AxonServer](https://axoniq.io/product-overview/axon-server).
This project is a sibling of [archetype-go-axon](https://github.com/dendrite2go/archetype-go-axon), but for the Rust programming language.

# Quick start

Requirements: Mac OS / X or Linux + docker.

Run
```shell
$ bin/clobber-build-and-run.sh
```
The first time this takes very long, because all dependencies have to be downloaded and compiled, both for the back-end and for the front-end. Subsequent runs will be much faster.

Wait until everything is available (ElasticSearch is usually last). When everything is up-and-running there are a few exposed ports that provide access to the various components of te example application:
* HTTP port 3000: [Front-end through proxy](http://localhost:3000)
* HTTP port 8024: [AxonServer](http://localhost:8024)
* gRPC port 8181: Back-end directly
* REST port 9200: ElasticSearch

# Core concepts

* [Command / Query Responsibility Segregation](http://codebetter.com/gregyoung/2010/02/16/cqrs-task-based-uis-event-sourcing-agh/) (CQRS)
* [Domain-driven Design](https://dddcommunity.org/learning-ddd/what_is_ddd/) (DDD)
* [Event Sourcing](https://axoniq.io/resources/event-sourcing)
* [Futures and async/await](https://rust-lang.github.io/async-book)
* [gRPC](https://grpc.io/)
* [Microservices](https://en.wikipedia.org/wiki/Microservices)
* [Service Mesh](https://buoyant.io/2017/04/25/whats-a-service-mesh-and-why-do-i-need-one/) (video: [talk by Jeroen Reijn](https://2019.jfall.nl/sessions/whats-a-service-mesh-and-why-do-i-need-one/))

# Design

Both the gRPC API that is exposed to the front-end and the payloads of the messages that are exchanged with AxonServer are defined using [Protocol Buffers](https://developers.google.com/protocol-buffers). This includes Commands, Command Projections, Command Results, Events, Queries, Query Results. (See [grpc_example.proto](https://github.com/dendrite2go/archetype-rust-axon/blob/master/proto/grpc_example.proto) for an example.) As events are stored in AxonServer indefinitely, extra attention and effort is advisable to evolve the definitions of events carefully. Introduce new types of events rather than changing existing ones, if possible. Make changes that are both backwards and forwards compatible if existing events need to be changed. Test with lots of data and rebuild all query models when changing event definitions.

A CQRS application based on AxonServer consists of loosely coupled parts that can be easily separated into microservices and that can be independently and horizontally scaled at will. There are five types of parts:

1. Command API
2. Command Processor
3. Event Processor
4. Query API
5. Query Processor

To start with it is advisable to combine all these components in a single back-end application (a structured monolith). The example application is also structured like this. When the application grows, parts can be separated out into different microservices as needed.

The responsibility of the Command API is to accept commands and submit them to AxonServer. Command API and Query API are often combined, but it is possible to define them as separate services in the proto definition and to create dedicated microservices for them. When the application grows, it also makes sense to split the API according to DDD Bounded Contexts.

A Command Processor is what is called the Aggregate in AxonFramework. It defines how the application responds to commands. It maintains a command projection for each aggregate and verifies incoming commands against the latest state of that projection. If a command is accepted, the handler can emit events that are stored in the Event Store of AxonServer. These events are also used to update the aggregate projection by applying event sourcing handlers. Normally the success response of a command is empty, but sometimes, for example when a new aggregate is created, it is desirable to return some value, _e.g._, the ID of the new aggregate. Errors in the Command Handler are propagated to the caller. Normally an aggregate coincides with a DDD Bounded context. In large systems each aggregate type will have its own microservice for the Command Handler. 

An Event Processor applies incoming events to a Query Model. Query models are not persisted in AxonServer, but in a suitable storage facility that is optimized for a particular type of queries. A token that indicates the last processed event is stored with the query model. If a query model is deleted, it will be automatically rebuilt fom the events. Event handlers should not fail. If an event handler fails, the entire event processor for that query model stops and has to be restarted when the problem has been fixed. There can be many query models, and they can even depend on events from different aggregate types. Each query model can have its own microservice.

The Query API accepts queries and submits them to AxonServer. The design considerations are very similar to the Command API. The query API can evolve much faster than the command API, because it follows User Experience demands, and it is much less constrained by business rules than the command API.

A Query Processor executes queries on a particular (type of) query model(s). It is advisable to keep query processors as simple as possible. Try to shift as much processing to event processors so that it can be done beforehand instead of making the client wait for it.

AxonServer guarantees that commands and events are processed sequentially for each aggregate, so there is no need for a transactional database to store aggregate state or query models.

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

# Usage

Use this project as a template for new Event Sourced CQRS projects with AxonServer by clicking the "Use this template" button of GitHub.

The file `etc/settings-sample.sh` contains sample settings for the project. It is advised to change at least the variable `ENSEMBLE_NAME` _before_ running `clobber-build-and-run.sh` when creating a new project from the template. The script `clobber-build-and-run.sh` automatically creates `etc/settings-local.sh` from `etc/settings-sample.sh` if it is not present. The file `etc/settings-local.sh` is ignored by git, so your local settings stay local.

The script `clobber-build-and-run.sh` takes the following arguments (options only work in the given order; when switched around the behavior is undefined):

|option|description
|------|-----------
|`-v`|once or twice for debug or trace mode respectively
|`--help`|show a brief usage reminder
|`--tee <file>`|write the output also to the given file
|`--skip-build`|skip the build phase, only clobber and run
|`--build-uses-siblings`|expose the parent of the project to the Rust compiler,<br/>so that the build can reference sibling projects<br/>(_e.g._, for testing changes to the dendrite library)
|`--back-end-only`|build only the back-end, not the front-end<br/>(most useful with `--dev`)
|`--no-clobber`|skip removal of the docker volumes where the data is kept
|`--dev`|use the development version of the front-end<br/>React will automatically recompile changes to front-end code

The file `etc/docker-compose.yml` is recreated from `etc/docker-compose-template.yml` by script `docker-compose-up.sh` for each run of `clobber-build-and-run.sh`.

The `--dev` mode deploys a `node.js` container that serves the sources directly from the `present` directory. React will dynamically recompile and reload sources when they are changed. Otherwise, an `nginx` container is deployed that serves the 'production' generated front-end.

There is a separate script `bin/docker-compose-up.sh` that only regenerates the `docker-compose.yml` and invokes docker compose up. It only takes options `-v1 (once or twice)` and `--dev`.

There is also a basic script `grpcurl-call.sh` that provides access to the gRPC API of the back-end from the command-line.