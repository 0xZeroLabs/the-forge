// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import "@storyprotocol/core/registries/IPAssetRegistry.sol";
import {PILFlavors} from "@storyprotocol/core/lib/PILFlavors.sol";
import {ISPGNFT} from "@storyprotocol/periphery/interfaces/ISPGNFT.sol";
import {RegistrationWorkflows} from "@storyprotocol/periphery/workflows/RegistrationWorkflows.sol";
import {WorkflowStructs} from "@storyprotocol/periphery/lib/WorkflowStructs.sol";
import {PILicenseTemplate} from "@storyprotocol/core/modules/licensing/PILicenseTemplate.sol";
import {ILicensingModule} from "@storyprotocol/core/interfaces/modules/licensing/ILicensingModule.sol";
import {PILTerms} from "@storyprotocol/core/interfaces/modules/licensing/IPILicenseTemplate.sol";
import {RoyaltyPolicyLAP} from "@storyprotocol/core/modules/royalty/policies/LAP/RoyaltyPolicyLAP.sol";
import {ForgeStorage} from "./ForgeStorage.sol";

contract ForgeRegistry is
    Initializable,
    OwnableUpgradeable,
    UUPSUpgradeable,
    ReentrancyGuardUpgradeable,
    ForgeStorage
{
    // Constants
    uint256 public constant UNLOCK_BLOCK_TIME = 3600 seconds;

    // State variables
    IPAssetRegistry public IP_ASSET_REGISTRY;
    RegistrationWorkflows public REGISTRATION_WORKFLOWS;
    PILicenseTemplate internal PIL_TEMPLATE;
    RoyaltyPolicyLAP internal ROYALTY_POLICY_LAP;
    ILicensingModule internal LICENSING_MODULE;

    // Structs
    struct IPMetadata {
        string name;
        string ipMetadataURI;
        string ipMetadata;
        string nftMetadataURI;
        string nftMetadata;
    }

    struct Terms {
        bool transferable;
        bool commercial;
        bool commercialAttribution;
    }

    // Events
    event PaymentReceived(address indexed sender, uint256 indexed amount);
    event FundsWithdrawn(address indexed recipient, uint256 indexed amount);
    event BalanceLocked(address indexed user);
    event BalanceUnlocked(address indexed user, uint256 unlockBlockTime);
    event IPRegistered(
        address indexed ipId,
        uint256 indexed tokenId,
        address indexed owner,
        string ipMetadataURI,
        string nftMetadataURI,
        string appId
    );

    // Errors
    error OnlyBatcherAllowed(address caller); // 152bc288
    error NoSubmitter(); // c43ac290
    error UserHasNoFundsToUnlock(address user); // b38340cf
    error UserHasNoFundsToLock(address user); // 6cc12bc2
    error PayerInsufficientBalance(uint256 balance, uint256 amount); // 21c3d50f
    error FundsLocked(uint256 unlockBlockTime, uint256 currentBlockTime); // bedc4e5a
    error SubmissionInsufficientBalance(
        address sender,
        uint256 balance,
        uint256 required
    ); // 4f779ceb
    error InvalidAddress(string param); // 161eb542

    // Modifiers
    modifier onlyBatcher() {
        if (msg.sender != batcherWallet) {
            revert OnlyBatcherAllowed(msg.sender);
        }
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    // Initialization and upgrade functions
    function initialize(
        address ipAssetRegistryAddress,
        address registrationWorkflowsAddress,
        address piLicenseTemplateAddress,
        address royaltyPolicyLAPAddress,
        address licensingModuleAddress,
        address owner,
        address _batcherWallet
    ) public initializer {
        if (ipAssetRegistryAddress == address(0)) {
            revert InvalidAddress("ipAssetRegistryAddress");
        }
        if (registrationWorkflowsAddress == address(0)) {
            revert InvalidAddress("registrationWorkflowsAddress");
        }
        if (piLicenseTemplateAddress == address(0)) {
            revert InvalidAddress("piLicenseTemplateAddress");
        }
        if (royaltyPolicyLAPAddress == address(0)) {
            revert InvalidAddress("royaltyPolicyLAPAddress");
        }
        if (licensingModuleAddress == address(0)) {
            revert InvalidAddress("licensingModuleAddress");
        }
        if (_batcherWallet == address(0)) {
            revert InvalidAddress("batcherWallet");
        }

        __Ownable_init(owner);
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        batcherWallet = _batcherWallet;

        IP_ASSET_REGISTRY = IPAssetRegistry(ipAssetRegistryAddress);
        REGISTRATION_WORKFLOWS = RegistrationWorkflows(
            registrationWorkflowsAddress
        );

        PIL_TEMPLATE = PILicenseTemplate(piLicenseTemplateAddress);
        ROYALTY_POLICY_LAP = RoyaltyPolicyLAP(royaltyPolicyLAPAddress);
        LICENSING_MODULE = ILicensingModule(licensingModuleAddress);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    // Core logic functions
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
        string memory appId,
        address submitter
    )
        public
        onlyBatcher
        nonReentrant
        returns (
            address ipId,
            uint256 tokenId,
            address owner,
            string memory ipMetadataURI,
            string memory nftMetadataURI
        )
    {
        require(receiver != address(0), "Invalid receiver address");
        require(bytes(_ipMetadata.name).length > 0, "Empty name");
        require(submitter != address(0), "Invalid submitter address");

        uint256 startGas = gasleft();

        ISPGNFT spgNft = _createNFTCollection(_ipMetadata.name, receiver);

        (ipId, tokenId) = REGISTRATION_WORKFLOWS.mintAndRegisterIp(
            address(spgNft),
            receiver,
            _generateIPMetadata(_ipMetadata)
        );
        owner = receiver;

        ipMetadataURI = _ipMetadata.ipMetadataURI;
        nftMetadataURI = _ipMetadata.nftMetadataURI;

        PILTerms memory pilTerms = PILFlavors.creativeCommonsAttribution({
            mintingFee: 0,
            royaltyPolicy: ROYALTY_POLICY_LAP
        });
        uint256 licenseTermsId = PIL_TEMPLATE.registerLicenseTerms(pilTerms);
        LICENSING_MODULE.attachLicenseTerms(
            ipId,
            address(PIL_TEMPLATE),
            licenseTermsId
        );

        uint256 gasUsed = startGas - gasleft();
        uint256 refundAmount = (gasUsed * tx.gasprice * 110) / 100;

        require(
            address(this).balance >= refundAmount,
            "Insufficient contract balance"
        );

        if (userData[submitter].balance < refundAmount) {
            revert SubmissionInsufficientBalance(
                submitter,
                userData[submitter].balance,
                refundAmount
            );
        }

        userData[submitter].nonce++;
        userData[submitter].balance -= refundAmount;

        emit IPRegistered(
            ipId,
            tokenId,
            receiver,
            _ipMetadata.ipMetadataURI,
            _ipMetadata.nftMetadataURI,
            appId
        );
        payable(msg.sender).transfer(refundAmount);
    }

    // Payment handling functions
    receive() external payable nonReentrant {
        userData[msg.sender].balance += msg.value;
        userData[msg.sender].unlockBlockTime = 0;
        emit PaymentReceived(msg.sender, msg.value);
    }

    function unlock() external nonReentrant {
        if (userData[msg.sender].balance == 0) {
            revert UserHasNoFundsToUnlock(msg.sender);
        }

        userData[msg.sender].unlockBlockTime =
            block.timestamp +
            UNLOCK_BLOCK_TIME;
        emit BalanceUnlocked(msg.sender, userData[msg.sender].unlockBlockTime);
    }

    function lock() external nonReentrant {
        if (userData[msg.sender].balance == 0) {
            revert UserHasNoFundsToLock(msg.sender);
        }
        userData[msg.sender].unlockBlockTime = 0;
        emit BalanceLocked(msg.sender);
    }

    function withdraw(uint256 amount) external nonReentrant {
        UserInfo storage senderData = userData[msg.sender];
        if (senderData.balance < amount) {
            revert PayerInsufficientBalance(senderData.balance, amount);
        }

        if (
            senderData.unlockBlockTime == 0 ||
            senderData.unlockBlockTime > block.timestamp
        ) {
            revert FundsLocked(senderData.unlockBlockTime, block.timestamp);
        }

        senderData.balance -= amount;
        senderData.unlockBlockTime = 0;
        payable(msg.sender).transfer(amount);
        emit FundsWithdrawn(msg.sender, amount);
    }

    // View functions
    function user_balances(address account) public view returns (uint256) {
        return userData[account].balance;
    }

    function user_nonces(address account) public view returns (uint256) {
        return userData[account].nonce;
    }

    function user_unlock_block(address account) public view returns (uint256) {
        return userData[account].unlockBlockTime;
    }
}
