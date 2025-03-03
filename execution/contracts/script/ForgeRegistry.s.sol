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
            vm.envAddress("PILICENSE_TEMPLATE_ADDRESS"),
            vm.envAddress("ROYALTY_POLICY_LAP_ADDRESS"),
            vm.envAddress("LICENSING_MODULE_ADDRESS"),
            deployer,
            vm.envAddress("BATCHER_ADDRESS")
        );

        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), data);

        ForgeRegistry(payable(address(proxy)));

        vm.stopBroadcast();

        console.log("Implementation deployed to:", address(implementation));
        console.log("Proxy deployed to:", address(proxy));
    }
}

contract UpgradeForgeRegistry is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address proxyAddress = vm.envAddress("PROXY_ADDRESS");
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        // Deploy new implementation
        ForgeRegistry newImplementation = new ForgeRegistry();

        bytes memory data = abi.encodeWithSelector(
            ForgeRegistry.reinitialize.selector,
            vm.envAddress("IP_ASSET_REGISTRY_ADDRESS"),
            vm.envAddress("REGISTRATION_WORKFLOWS_ADDRESS"),
            vm.envAddress("PILICENSE_TEMPLATE_ADDRESS"),
            vm.envAddress("ROYALTY_POLICY_LAP_ADDRESS"),
            vm.envAddress("LICENSING_MODULE_ADDRESS"),
            deployer,
            vm.envAddress("BATCHER_ADDRESS")
        );
        // Upgrade and call the reinitializer
        UUPSUpgradeable(proxyAddress).upgradeToAndCall(
            address(newImplementation),
            data
        );

        vm.stopBroadcast();

        console.log(
            "New implementation deployed to:",
            address(newImplementation)
        );
        console.log("Proxy upgraded at:", proxyAddress);
    }
}
