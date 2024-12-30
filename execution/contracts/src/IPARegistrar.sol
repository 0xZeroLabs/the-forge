// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import {IPAssetRegistry} from "@storyprotocol/core/registries/IPAssetRegistry.sol";
import {ISPGNFT} from "@storyprotocol/periphery/interfaces/ISPGNFT.sol";
import {RegistrationWorkflows} from "@storyprotocol/periphery/workflows/RegistrationWorkflows.sol";
import {WorkflowStructs} from "@storyprotocol/periphery/lib/WorkflowStructs.sol";

contract IPARegistrar {
    IPAssetRegistry public immutable IP_ASSET_REGISTRY;
    RegistrationWorkflows public immutable REGISTRATION_WORKFLOWS;

    constructor(
        address ipAssetRegistryAddress,
        address registrationWorkflowsAddress
    ) {
        IP_ASSET_REGISTRY = IPAssetRegistry(ipAssetRegistryAddress);
        REGISTRATION_WORKFLOWS = RegistrationWorkflows(
            registrationWorkflowsAddress
        );
    }

    struct IPMetadata {
        string name;
        string ipMetadataURI;
        string ipMetadata;
        string nftMetadataURI;
        string nftMetadata;
    }

    function _createNFTCollection(
        string memory name,
        address owner
    ) private returns (ISPGNFT) {
        return
            ISPGNFT(
                REGISTRATION_WORKFLOWS.createCollection(
                    ISPGNFT.InitParams({
                        name: name,
                        symbol: "IPA",
                        baseURI: "",
                        contractURI: "",
                        maxSupply: 100,
                        mintFee: 0,
                        mintFeeToken: address(0),
                        mintFeeRecipient: owner,
                        owner: owner,
                        mintOpen: true,
                        isPublicMinting: false
                    })
                )
            );
    }

    function _generateIPMetadata(
        IPMetadata memory metadata
    ) private pure returns (WorkflowStructs.IPMetadata memory) {
        return
            WorkflowStructs.IPMetadata({
                ipMetadataURI: metadata.ipMetadataURI,
                ipMetadataHash: keccak256(
                    abi.encodePacked(metadata.ipMetadata)
                ),
                nftMetadataURI: metadata.nftMetadataURI,
                nftMetadataHash: keccak256(
                    abi.encodePacked(metadata.nftMetadata)
                )
            });
    }

    function register(
        address _address,
        IPMetadata memory _ipMetadata
    ) public returns (address) {
        ISPGNFT spgNft = _createNFTCollection(_ipMetadata.name, _address);

        (address ipId, ) = REGISTRATION_WORKFLOWS.mintAndRegisterIp(
            address(spgNft),
            _address,
            _generateIPMetadata(_ipMetadata)
        );

        return ipId;
    }
}
