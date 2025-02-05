// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ForgeRegistry} from "../src/ForgeRegistry.sol";

contract ForgeRegistryScript is Script {
    ForgeRegistry public registrar;

    function setUp() public {}

    function run() public {
        // Get deployment addresses from environment variables
        address ipAssetRegistryAddress = vm.envAddress(
            "IP_ASSET_REGISTRY_ADDRESS"
        );
        address registrationWorkflowsAddress = vm.envAddress(
            "REGISTRATION_WORKFLOWS_ADDRESS"
        );

        vm.startBroadcast();

        registrar = new ForgeRegistry(
            ipAssetRegistryAddress,
            registrationWorkflowsAddress
        );

        vm.stopBroadcast();

        console.log("ForgeRegistry deployed at:", address(registrar));
    }
}
