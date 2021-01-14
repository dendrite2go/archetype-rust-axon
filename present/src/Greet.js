import React, { Component } from 'react';
import example from './grpc/example/grpc_example_grpc_web_pb';
// import example_message from './grpc/example/grpc_example_pb';

class Greet extends Component {
    constructor(props) {
        super(props);
        this.handleSubmit = this.handleSubmit.bind(this);
        this.handleQuery = this.handleQuery.bind(this);
        this.handleRefresh = this.handleRefresh.bind(this);
        this.handleRecord = this.handleRecord.bind(this);
        this.handleStop = this.handleStop.bind(this);
        console.log('Example grpc-web stub:', example);
    }

    render() {
        return (
            <div>
                <h3>Greetings</h3>
                <p><input type='submit' id='record' value=' Record ' onClick={this.handleRecord}/>
                   <input type='submit' id='stop' value=' Stop ' onClick={this.handleStop} className='trailing'/>
                </p>
                <p><input type='text' id='message' />
                   <input type='submit' id='submit-greeting' value=' Go! ' onClick={this.handleSubmit} className='trailing'/>
                   <input type='submit' id='submit-query' value=' Submit Lucene query ' onClick={this.handleQuery} className='trailing'/>
                </p>
                <p><input type='submit' id='refresh-greetings' value=' Refresh! ' onClick={this.handleRefresh}/></p>
                <div id='greetings'><div><i>greetings appear here</i></div></div>
            </div>
        );
    }

    handleSubmit(event) {
        const message = document.getElementById('message').value;
        console.log('Submit: message:', message);
        const request = new example.Greeting();
        console.log('Submit: new request:', request);
        request.setMessage(message);
        console.log('Submit: request:', request);
        const client = new example.GreeterServiceClient('http://localhost:3000');
        console.log('Submit: client:', client);
        const response = client.greet(request);
        console.log('Submit: response:', response);
        response.on('data', function(r) {console.log('Greet event:', r);})
        response.on('status', function(status) {
          console.log('Submit: stream status: code:', status.code);
          console.log('Submit: stream status: details:', status.details);
          console.log('Submit: stream status: metadata:', status.metadata);
        });
    }

    handleQuery(event) {
        const query = document.getElementById('message').value;
        console.log('Query: query:', query);

        const client = new example.GreeterServiceClient('http://localhost:3000');
        console.log('Query: client:', client);

        const container = document.getElementById('greetings');
        container.innerHTML = '';

        const searchQueryRequest = new example.SearchQuery();
        console.log('Query: new request:', searchQueryRequest);
        searchQueryRequest.setQuery(query);
        console.log('Query: request:', searchQueryRequest);
        const response = client.search(searchQueryRequest);

        console.log('Query: response:', response);
        response.on('data', function(r) {
            console.log('Query: greetings document:', r);
            const message = r.getMessage();
            console.log('Query: greetings document: message', message);
            const text = document.createTextNode(message);
            const div = document.createElement('div');
            div.appendChild(text);
            container.appendChild(div)
        })
        response.on('status', function(status) {
          console.log('Query: stream status: code:', status.code);
          console.log('Query: stream status: details:', status.details);
          console.log('Query: stream status: metadata:', status.metadata);
        });
        response.on('end', function(end) {
          // stream end signal
        });
    }

    handleRefresh(event) {
        console.log('Handle refresh:', event)

        const client = new example.GreeterServiceClient('http://localhost:3000');
        console.log('Refresh: client:', client);

        const container = document.getElementById('greetings');
        container.innerHTML = '';

        const request = new example.Empty();
        console.log('Refresh: new request:', request);
        const response = client.greetings(request);

        console.log('Refresh: response:', response);
        response.on('data', function(r) {
            console.log('Refresh: greetings event:', r);
            const message = r.getMessage();
            console.log('Refresh: greetings event: message', message);
            const text = document.createTextNode(message);
            const div = document.createElement('div');
            div.appendChild(text);
            container.appendChild(div)
        })
        response.on('status', function(status) {
          console.log('Refresh: stream status: code:', status.code);
          console.log('Refresh: stream status: details:', status.details);
          console.log('Refresh: stream status: metadata:', status.metadata);
        });
        response.on('end', function(end) {
          // stream end signal
        });
    }

    handleRecord(event) {
        const request = new example.Empty();
        console.log('Record: new request:', request);
        const client = new example.GreeterServiceClient('http://localhost:3000');
        console.log('Record: client:', client);
        const response = client.record(request);
        console.log('Record: response:', response);
        response.on('data', function(r) {console.log('Record response data:', r);})
        response.on('status', function(status) {
          console.log('Record: stream status: code:', status.code);
          console.log('Record: stream status: details:', status.details);
          console.log('Record: stream status: metadata:', status.metadata);
        });
    }

    handleStop(event) {
        const request = new example.Empty();
        console.log('Stop: new request:', request);
        const client = new example.GreeterServiceClient('http://localhost:3000');
        console.log('Stop: client:', client);
        const response = client.stop(request);
        console.log('Stop: response:', response);
        response.on('data', function(r) {console.log('Stop response data:', r);})
        response.on('status', function(status) {
          console.log('Stop: stream status: code:', status.code);
          console.log('Stop: stream status: details:', status.details);
          console.log('Stop: stream status: metadata:', status.metadata);
        });
    }
}

export default Greet;
