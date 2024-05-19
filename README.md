# deployer-of-worlds

A config driven smart contract deployment and execution framework for repeatable to deterministic actions.

## Example Config

```
[
	{
		"id": "deploy_contract",
		"depends_on": [],
		"action_data": {
			"type": "deploy",
			"content": {
				"address": "0x854914dA8b451F82eE2CD08E43116Ae20Eb2EdC9",
				"salt": 1,
				"bytecode": "0x0000000000000000000000000000000000000000000000000000000000000001",
				"abi":   {
			    "type": "constructor",
			    "payable": false,
			    "inputs": [
			      { "type": "string", "name": "symbol" },
			      { "type": "string", "name": "name" }
			    ]
			  },
				"constructor_args": ["0x0", "0x0"]
			}
		},
		"output_schema": {
			"output_type": "string"
		}
	},
	{
		"id": "stake_contract",
		"depends_on": ["deploy_contract"],
		"inputs": ["deploy_contract"],
		"action_data": {
			"type": "write",
			"content": {
				"address": "inputs.deploy_contract",
				"abi":   {
			    "type": "constructor",
			    "payable": false,
			    "inputs": [
			      { "type": "string", "name": "symbol" },
			      { "type": "string", "name": "name" }
			    ]
			  },
				"args": ["0x0", "0x0"],
				"value": 0
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
