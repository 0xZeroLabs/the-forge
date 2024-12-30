// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {IPARegistrar} from "../src/IPARegistrar.sol";

contract IPARegistrarScript is Script {
    IPARegistrar public registrar;

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

        registrar = new IPARegistrar(
            ipAssetRegistryAddress,
            registrationWorkflowsAddress
        );

        vm.stopBroadcast();

        console.log("IPARegistrar deployed at:", address(registrar));
    }
}
