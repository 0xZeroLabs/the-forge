// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ForgeRegistry} from "../src/ForgeRegistry.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

contract DeployForgeRegistry is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);

        ForgeRegistry implementation = new ForgeRegistry();

        bytes memory data = abi.encodeWithSelector(
            ForgeRegistry.initialize.selector,
            vm.envAddress("IP_ASSET_REGISTRY_ADDRESS"),
            vm.envAddress("REGISTRATION_WORKFLOWS_ADDRESS"),
            deployer
        );

        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), data);

        ForgeRegistry(address(proxy));

        vm.stopBroadcast();

        console.log("Implementation deployed to:", address(implementation));
        console.log("Proxy deployed to:", address(proxy));
    }
}

contract UpgradeForgeRegistry is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address proxyAddress = vm.envAddress("PROXY_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);

        // Deploy new implementation
        ForgeRegistry newImplementation = new ForgeRegistry();

        // Upgrade
        UUPSUpgradeable(proxyAddress).upgradeToAndCall(
            address(newImplementation),
            "" // No need to call initialize again
        );

        vm.stopBroadcast();

        console.log(
            "New implementation deployed to:",
            address(newImplementation)
        );
        console.log("Proxy upgraded at:", proxyAddress);
    }
}
