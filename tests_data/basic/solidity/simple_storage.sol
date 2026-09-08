// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SimpleStorage
 * @dev Store and retrieve value in a variable
 */
contract SimpleStorage {
    uint256 private storedData;
    address public immutable owner;

    event ValueChanged(address indexed author, uint256 newValue);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the contract owner can perform this operation");
        _;
    }

    constructor(uint256 initialValue) {
        owner = msg.sender;
        storedData = initialValue;
        emit ValueChanged(msg.sender, initialValue);
    }

    function set(uint256 x) public onlyOwner {
        storedData = x;
        emit ValueChanged(msg.sender, x);
    }

    function get() public view returns (uint256) {
        return storedData;
    }
}
