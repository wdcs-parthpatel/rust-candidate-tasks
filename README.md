1. Simple Client
Build a simple client that can run the two following commands:
./simple --mode=cache --times=10
./simple --mode=read
The cache mode should listen to a websocket for given number of times(seconds) only for the USD prices of BTC. 
Example is given here https://binance-docs.github.io/apidocs/websocket_api/en/#symbol-price-ticker, any other websocket is also fine like kucoin, gemini, gateio, bybit etc. Calculate the average of these prices, say XXX. 
Then print "Cache complete. The average USD price of BTC is: XXX" 
Save both the result of the aggregate and the data points used to create the aggregate to a file.
The read mode should just read from the file and print the values to the terminal.

2. Simulated distributed client
Extend the solution to Q1 by instantiating 5 client processes and one aggregator process.
All client processes start at the same tick of the time, say 10:01:01 AM.
Client process read values from the websocket for 10 seconds and computes the average and sends it to the aggregator process.
Aggregator process waits for the average values from all the 5 processes. Upon getting all the values it computes yet another average and displays on the screen.

3. Using signatures
Extend the solution to Q2 where the clients send the signed messages to the aggregator. And the aggregator validates the signatures and then computes the average of averages. 
Any signature scheme is fine. Set up the premise such that all the processes knows the public keys of all other processes before hand.

4. Build a Networked Tic-Tac-Toe Game
Title: Develop a Multi-Player Networked Tic-Tac-Toe Game
Description: Implement a multiplayer Tic-Tac-Toe game where multiple players can connect to a server to play against each other in real time. The server manages the game state and ensures that each player makes valid moves. The client interacts with the server to send moves and receive game status updates.

5. Web3 Ethereum Wallet
Write a simple Web3 wallet in Rust that connects to an Ethereum testnet blockchain node using web3-rs.
Requirements
Generate a new Ethereum wallet with a private key.
Connect to an Ethereum node (Infura, Alchemy, or a local node).
Fetch balance for an address.
Send an ETH transaction to another wallet.

6. Create a rust binary
That allows you to continuously print the incoming tx of a particular address with amount

7. Smart Contract Event Listener

// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

/**
 * @title Storage
 * @dev Store & retrieve value in a variable
 * @custom:dev-run-script ./scripts/deploy_with_ethers.ts
 */
contract Storage {

    uint256 number;
    event NumberUpdatedEvent(address Sender);

    /**
     * @dev Store value in variable
     * @param num value to store
     */
    function store(uint256 num) public {
        number = num;
        emit NumberUpdatedEvent(msg.sender);
    }

    /**
     * @dev Return value 
     * @return value of 'number'
     */
    function retrieve() public view returns (uint256){
        return number;
    }
}

Now this is the SmartContract (you can deploy the same on a new address).
you need to create a rust binary which will listen to the event NumberUpdatedEvent(msg.sender)
and print out the tx and sender and block of the event and also the value after the event has occurred.


8. Create a benchmark 
Create a benchmark result for different types of serialization and deserialization in rust.
eg. bincode, bcs, protobuf, serde_json, borshe etc.
and compare various metrics of each
eg. sizes, compute time, etc.


9. Create a library
Create a library to generate a merkle tree of given leaves.
It should also give some interface to get proof, verify proof, and calculate root.
