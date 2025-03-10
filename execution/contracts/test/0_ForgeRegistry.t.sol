// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IPAssetRegistry} from "@storyprotocol/core/registries/IPAssetRegistry.sol";
import {ISPGNFT} from "@storyprotocol/periphery/interfaces/ISPGNFT.sol";
import {RegistrationWorkflows} from "@storyprotocol/periphery/workflows/RegistrationWorkflows.sol";
import {WorkflowStructs} from "@storyprotocol/periphery/lib/WorkflowStructs.sol";

import {SimpleNFT} from "../src/mocks/SimpleNFT.sol";

// Run this test:
// forge test --fork-url https://odyssey.storyrpc.io/ --match-path test/0_ForgeRegistry.t.sol
contract ForgeRegistryTest is Test {
    address internal alice = address(0xa11ce);

    // For addresses, see https://docs.story.foundation/docs/deployed-smart-contracts
    // Protocol Core - IPAssetRegistry
    IPAssetRegistry public immutable IP_ASSET_REGISTRY =
        IPAssetRegistry(0x28E59E91C0467e89fd0f0438D47Ca839cDfEc095);
    // Protocol Periphery - RegistrationWorkflows
    RegistrationWorkflows public immutable REGISTRATION_WORKFLOWS =
        RegistrationWorkflows(0xde13Be395E1cd753471447Cf6A656979ef87881c);

    SimpleNFT public SIMPLE_NFT;
    ISPGNFT public SPG_NFT;

    function setUp() public {
        // Create a new Simple NFT collection
        SIMPLE_NFT = new SimpleNFT("Simple IP NFT", "SIM");
        // Create a new NFT collection via SPG
        SPG_NFT = ISPGNFT(
            REGISTRATION_WORKFLOWS.createCollection(
                ISPGNFT.InitParams({
                    name: "Test Collection",
                    symbol: "TEST",
                    baseURI: "",
                    contractURI: "",
                    maxSupply: 100,
                    mintFee: 0,
                    mintFeeToken: address(0),
                    mintFeeRecipient: address(this),
                    owner: address(this),
                    mintOpen: true,
                    isPublicMinting: false
                })
            )
        );
    }

    /// @notice Mint an NFT and then register it as an IP Asset.
    function test_register() public {
        uint256 expectedTokenId = SIMPLE_NFT.nextTokenId();
        address expectedIpId = IP_ASSET_REGISTRY.ipId(
            block.chainid,
            address(SIMPLE_NFT),
            expectedTokenId
        );

        uint256 tokenId = SIMPLE_NFT.mint(alice);
        address ipId = IP_ASSET_REGISTRY.register(
            block.chainid,
            address(SIMPLE_NFT),
            tokenId
        );

        assertEq(tokenId, expectedTokenId);
        assertEq(ipId, expectedIpId);
        assertEq(SIMPLE_NFT.ownerOf(tokenId), alice);
    }

    /// @notice Mint an NFT and register it in the same call via the Story Protocol Gateway.
    /// @dev Requires the collection address that is passed into the `mintAndRegisterIp` function
    /// to be created via SPG (createCollection), as done above. Or, a contract that
    /// implements the `ISPGNFT` interface.
    function test_mintAndRegisterIp() public {
        uint256 expectedTokenId = SPG_NFT.totalSupply() + 1;
        address expectedIpId = IP_ASSET_REGISTRY.ipId(
            block.chainid,
            address(SPG_NFT),
            expectedTokenId
        );

        // Note: The caller of this function must be the owner of the SPG NFT Collection.
        // In this case, the owner of the SPG NFT Collection is the contract itself
        // because it deployed it in the `setup` function.
        // We can make `alice` the recipient of the NFT though, which makes her the
        // owner of not only the NFT, but therefore the IP Asset.
        (address ipId, uint256 tokenId) = REGISTRATION_WORKFLOWS
            .mintAndRegisterIp(
                address(SPG_NFT),
                alice,
                WorkflowStructs.IPMetadata({
                    ipMetadataURI: "https://ipfs.io/ipfs/QmZHfQdFA2cb3ASdmeGS5K6rZjz65osUddYMURDx21bT73",
                    ipMetadataHash: keccak256(
                        abi.encodePacked(
                            "{'title':'My IP Asset','description':'This is a test IP asset','ipType':'','relationships':[],'createdAt':'','watermarkImg':'https://picsum.photos/200','creators':[],'media':[],'attributes':[{'key':'Rarity','value':'Legendary'}],'tags':[]}"
                        )
                    ),
                    nftMetadataURI: "https://ipfs.io/ipfs/QmRL5PcK66J1mbtTZSw1nwVqrGxt98onStx6LgeHTDbEey",
                    nftMetadataHash: keccak256(
                        abi.encodePacked(
                            "{'name':'Test NFT','description':'This is a test NFT','image':'https://picsum.photos/200'}"
                        )
                    )
                }),
                true
            );

        assertEq(ipId, expectedIpId);
        assertEq(tokenId, expectedTokenId);
        assertEq(SPG_NFT.ownerOf(tokenId), alice);
    }
}
