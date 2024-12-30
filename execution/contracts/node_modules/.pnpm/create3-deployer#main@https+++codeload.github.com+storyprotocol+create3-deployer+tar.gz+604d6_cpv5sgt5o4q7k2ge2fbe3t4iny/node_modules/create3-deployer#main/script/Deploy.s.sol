// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import { Script } from "forge-std/Script.sol";
import "forge-std/console2.sol";
import { Create3Deployer } from "contracts/Create3Deployer.sol";

contract DeployScript is Script {
    function run() public {
        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
        bytes32 salt = "Story Protocol Create3 Deployer";
        address expectedAddr = vm.computeCreate2Address(salt, keccak256(type(Create3Deployer).creationCode));
        if (expectedAddr.code.length > 0) {
            console2.log("Create3Deployer already deployed at: ", expectedAddr);
            vm.stopBroadcast();
            return;
        }
        Create3Deployer deployer = new Create3Deployer{
            salt: salt
        }();
        console2.log("Create3Deployer: ", address(deployer));
        vm.stopBroadcast();
    }
}
