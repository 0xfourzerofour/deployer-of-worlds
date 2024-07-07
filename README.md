# deployer-of-worlds

A config driven smart contract deployment and execution framework for repeatable to deterministic actions.

## Example Config

```json
[
  {
    "id": "read_entrypoint_deposit_info",
    "action_data": {
      "type": "read",
      "content": {
        "address": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        "function_signature": "function balanceOf(address owner) view returns (uint256 balance)",
        "args": [
          "0x4f84a207A80c39E9e8BaE717c1F25bA7AD1fB08F"
        ]
      }
    }
  },
  {
    "id": "read_entrypoint_get_balance",
    "depends_on": ["read_entrypoint_deposit_info"],
    "action_data": {
      "type": "read",
      "content": {
        "address": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        "function_signature": "function balanceOf(address owner) view returns (uint balance)",
        "args": [
          "${read_entrypoint_deposit_info}"
        ]
      }
    }
  }
]

```


## TODO

- Input validation based on id's and output schema based on jq queries 
- Generate init code based on input abi and constructor args
- Conditional Execution logic based on on-chain read funcitonality
- CREATE2 Deployer implementation for contract deployment
- CLI tool to allow for arbitrary json inputs to be executed
- Use Create2 deployer factory and bind rust types to it (here)[https://github.com/pcaversaccio/createx]
