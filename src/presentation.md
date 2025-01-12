---
marp: true
theme: default
paginate: true
---

# Rust Challenge:  Orderbooks
#### by Ruben Stukken

---

# Background

* 10 years experience in Options Market Making.
* C++ Programming.
* Single Threaded Reactor Design Pattern.

---

# Reactor Design Pattern

Concurrent event handling using a Runtime.
* Event demultiplexer (e.g. epoll) for fd, sockets.
* Reactor (runtime for event handling).
* Event handlers (business logic).

---


# Comparision with Async Rust

| Aspect             | C++ Reactor      | Async Rust (Tokio)               |
|--------------------|-----------------------------------|-----------------------------------------|
| **Concurrency** | Event-driven, single thread   | Single or multi-thread     |
| **Features** | Callbacks and state machines     | Async/await futures, tasks     |
| **Ease of Use**    | Boilerplate, explicit I/O   | High-level abstractions |              |
| **Memory Safety**  | Manual memory management         | Guaranteed memory safety   |

---

# Design Approach

### Orderbook Processor
* Owns list of exchange websockets
* Maintains Combined book
* Publishes to Orderbook Service

### Orderbook Service
* Implements gRPC service
* Maintains client connections


---

# Design Approach
## Exchange Websocket
* Exchange specific WS initialisation, deserialisation.
* Common trait/interface used by Orderbook Processor.
* Implements a stream of orderbooks.

## Orderbook Processor
* Combines all exchange WS streams into 1.
* Triggers a combined orderbook update.
* Forwards combined book to the Ordebook Service.

---

## Combined Orderbook
* Implements business logic.
* Unit tested.

## Orderbook Server
* Implements gRPC service.
* Client connections, data publishing.

---

# Python Client
* Command-line tool
* Displays the combined orderbook.

 ---

# Reactor Design Pattern
## Reactor (i.e. Runtime)

```cpp
class Reactor {
public:

    using EventHandler = std::function<void()>;

    void registerHandler(int fd, EventHandler handler);
    void unregisterHandler(int fd);

    void run();

private:

    std::unordered_map<int, EventHandler> handlers;
};
````

---

# Reactor Design Pattern
## Application - Header file

```cpp
class Application {

public:

    Application(Reactor& reactor);
    void setup();

private:

    Reactor& reactor;

    // Example callbacks
    void onClientDataReady();
    void onServerDataReady();
};
```
---

# Reactor Design Pattern
## Application - Implementation

```cpp
Application::Application(Reactor& reactor) : reactor(reactor) {}

void Application::setup() {

    int clientFd = 3;
    int serverFd = 4;

    // Register handlers with the reactor
    reactor.registerHandler(clientFd, [this]() { onClientDataReady(); });
    reactor.registerHandler(serverFd, [this]() { onServerDataReady(); });
}
```

---
