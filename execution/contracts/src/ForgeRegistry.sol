// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import {IPAssetRegistry} from "@storyprotocol/core/registries/IPAssetRegistry.sol";
import {ISPGNFT} from "@storyprotocol/periphery/interfaces/ISPGNFT.sol";
import {RegistrationWorkflows} from "@storyprotocol/periphery/workflows/RegistrationWorkflows.sol";
import {WorkflowStructs} from "@storyprotocol/periphery/lib/WorkflowStructs.sol";

contract ForgeRegistry {
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

    event IPRegistered(
        address indexed ipId,
        uint256 indexed tokenId,
        address indexed owner,
        string ipMetadataURI,
        string nftMetadataURI,
        string appId
    );

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
                        symbol: "ForgeIPA",
                        baseURI: "",
                        contractURI: "",
                        maxSupply: 100,
                        mintFee: 0,
                        mintFeeToken: address(0),
                        mintFeeRecipient: owner,
                        owner: owner,
                        mintOpen: true,
                        isPublicMinting: true
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
        address receiver,
        IPMetadata memory _ipMetadata,
        string memory appId
    ) public returns (address ipId, uint256 tokenId, address owner) {
        ISPGNFT spgNft = _createNFTCollection(_ipMetadata.name, receiver);

        (ipId, tokenId) = REGISTRATION_WORKFLOWS.mintAndRegisterIp(
            address(spgNft),
            receiver,
            _generateIPMetadata(_ipMetadata)
        );
        owner = receiver;

        emit IPRegistered(
            ipId,
            tokenId,
            receiver,
            _ipMetadata.ipMetadataURI,
            _ipMetadata.nftMetadataURI,
            appId
        );
    }
}
