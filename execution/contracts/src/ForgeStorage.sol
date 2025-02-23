// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

abstract contract ForgeStorage {
    struct UserInfo {
        uint256 balance;
        uint256 unlockBlockTime;
        uint256 nonce;
    }

    address public batcherWallet;

    mapping(address => UserInfo) public userData;

    uint256[24] private __GAP;
}
