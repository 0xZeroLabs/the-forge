// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IPAssetRegistry} from "@storyprotocol/core/registries/IPAssetRegistry.sol";
import {ISPGNFT} from "@storyprotocol/periphery/interfaces/ISPGNFT.sol";
import {RegistrationWorkflows} from "@storyprotocol/periphery/workflows/RegistrationWorkflows.sol";
import {WorkflowStructs} from "@storyprotocol/periphery/lib/WorkflowStructs.sol";

contract ForgeRegistry is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    IPAssetRegistry public IP_ASSET_REGISTRY;
    RegistrationWorkflows public REGISTRATION_WORKFLOWS;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address ipAssetRegistryAddress,
        address registrationWorkflowsAddress,
        address owner
    ) public initializer {
        __Ownable_init(owner);
        __UUPSUpgradeable_init();

        IP_ASSET_REGISTRY = IPAssetRegistry(ipAssetRegistryAddress);
        REGISTRATION_WORKFLOWS = RegistrationWorkflows(
            registrationWorkflowsAddress
        );
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

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
                        symbol: "INGOT",
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
    )
        public
        returns (
            address ipId,
            uint256 tokenId,
            address owner,
            string memory ipMetadataURI,
            string memory nftMetadataURI
        )
    {
        ISPGNFT spgNft = _createNFTCollection(_ipMetadata.name, receiver);

        (ipId, tokenId) = REGISTRATION_WORKFLOWS.mintAndRegisterIp(
            address(spgNft),
            receiver,
            _generateIPMetadata(_ipMetadata)
        );
        owner = receiver;

        ipMetadataURI = _ipMetadata.ipMetadataURI;
        nftMetadataURI = _ipMetadata.nftMetadataURI;

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
