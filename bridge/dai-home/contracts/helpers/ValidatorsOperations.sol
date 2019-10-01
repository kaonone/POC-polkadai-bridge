/*
  License: MIT
  Copyright Bitclave, 2018
  It's modified contract ValidatorOperations from https://github.com/bitclave/ValidatorOperations
*/

pragma solidity ^0.5.9;

import "openzeppelin-solidity/contracts/math/SafeMath.sol";

contract ValidatorsOperations {

    using SafeMath for uint256;

    using SafeMath for uint8;
    // VARIABLES

    uint256 public validatorsGeneration;
    uint256 public howManyValidatorsDecide;
    address[] public validators;
    bytes32[] public allOperations;
    address internal insideCallSender;
    uint256 internal insideCallCount;
    

    // Reverse lookup tables for validators and allOperations
    mapping(address => uint8) public validatorsIndices; // Starts from 1, size 255
    mapping(bytes32 => uint) public allOperationsIndicies;
    

    // validators voting mask per operations
    mapping(bytes32 => uint256) public votesMaskByOperation;
    mapping(bytes32 => uint256) public votesCountByOperation;

    //operation -> ValidatorIndex
    mapping(bytes32 => uint8) internal  operationsByValidatorIndex;
    mapping(uint8 => uint8) internal operationsCountByValidatorIndex;
    // EVENTS

    event ValidatorShipTransferred(address[] previousValidators, uint howManyValidatorsDecide, address[] newvalidators, uint newHowManyValidatorsDecide);
    event OperationCreated(bytes32 operation, uint howMany, uint validatorsCount, address proposer);
    event OperationUpvoted(bytes32 operation, uint votes, uint howMany, uint validatorsCount, address upvoter);
    event OperationPerformed(bytes32 operation, uint howMany, uint validatorsCount, address performer);
    event OperationDownvoted(bytes32 operation, uint votes, uint validatorsCount,  address downvoter);
    event OperationCancelled(bytes32 operation, address lastCanceller);
    
    // ACCESSORS

    function isExistValidator(address wallet) public view returns(bool) {
        return validatorsIndices[wallet] > 0;
    }


    function validatorsCount() public view returns(uint) {
        return validators.length;
    }

    function allOperationsCount() public view returns(uint) {
        return allOperations.length;
    }

    /*
      Internal functions
    */

    function _operationLimitByValidatorIndex(uint8 ValidatorIndex) internal view returns(bool) {
        return (operationsCountByValidatorIndex[ValidatorIndex] <= 3);
    }
    
    function _cancelAllPending() internal {
        for (uint i = 0; i < allOperations.length; i++) {
            delete(allOperationsIndicies[allOperations[i]]);
            delete(votesMaskByOperation[allOperations[i]]);
            delete(votesCountByOperation[allOperations[i]]);
            //delete operation->ValidatorIndex
            delete(operationsByValidatorIndex[allOperations[i]]);
        }

        allOperations.length = 0;
        //delete operations count for Validator
        for (uint8 j = 0; j < validators.length; j++) {
            operationsCountByValidatorIndex[j] = 0;
        }
    }


    // MODIFIERS

    /**
    * @dev Allows to perform method by any of the validators
    */
    modifier onlyAnyValidator {
        if (checkHowManyValidators(1)) {
            bool update = (insideCallSender == address(0));
            if (update) {
                insideCallSender = msg.sender;
                insideCallCount = 1;
            }
            _;
            if (update) {
                insideCallSender = address(0);
                insideCallCount = 0;
            }
        }
    }

    /**
    * @dev Allows to perform method only after many validators call it with the same arguments
    */
    modifier onlyManyValidators {
        if (checkHowManyValidators(howManyValidatorsDecide)) {
            bool update = (insideCallSender == address(0));
            if (update) {
                insideCallSender = msg.sender;
                insideCallCount = howManyValidatorsDecide;
            }
            _;
            if (update) {
                insideCallSender = address(0);
                insideCallCount = 0;
            }
        }
    }

    /**
    * @dev Allows to perform method only after all validators call it with the same arguments
    */
    modifier onlyAllValidators {
        if (checkHowManyValidators(validators.length)) {
            bool update = (insideCallSender == address(0));
            if (update) {
                insideCallSender = msg.sender;
                insideCallCount = validators.length;
            }
            _;
            if (update) {
                insideCallSender = address(0);
                insideCallCount = 0;
            }
        }
    }

    /**
    * @dev Allows to perform method only after some validators call it with the same arguments
    */
    modifier onlySomeValidators(uint howMany) {
        require(howMany > 0, "onlySomevalidators: howMany argument is zero");
        require(howMany <= validators.length, "onlySomevalidators: howMany argument exceeds the number of validators");
        
        if (checkHowManyValidators(howMany)) {
            bool update = (insideCallSender == address(0));
            if (update) {
                insideCallSender = msg.sender;
                insideCallCount = howMany;
            }
            _;
            if (update) {
                insideCallSender = address(0);
                insideCallCount = 0;
            }
        }
    }

    // CONSTRUCTOR

    constructor() public {
        validators.push(msg.sender);
        validatorsIndices[msg.sender] = 1;
        howManyValidatorsDecide = 1;
    }

    // INTERNAL METHODS

    /**
     * @dev onlyManyvalidators modifier helper
     */
    function checkHowManyValidators(uint howMany) internal returns(bool) {
        if (insideCallSender == msg.sender) {
            require(howMany <= insideCallCount, "checkHowManyvalidators: nested validators modifier check require more Validators");
            return true;
        }
        
        
        require((isExistValidator(msg.sender) && (validatorsIndices[msg.sender] <= validators.length)), "checkHowManyvalidators: msg.sender is not an Validator");

        uint ValidatorIndex = validatorsIndices[msg.sender].sub(1);
        
        bytes32 operation = keccak256(abi.encodePacked(msg.data, validatorsGeneration));

        require((votesMaskByOperation[operation] & (2 ** ValidatorIndex)) == 0, "checkHowManyvalidators: Validator already voted for the operation");
        //check limit for operation
        require(_operationLimitByValidatorIndex(uint8(ValidatorIndex)), "checkHowManyvalidators: operation limit is reached for this Validator");

        votesMaskByOperation[operation] |= (2 ** ValidatorIndex);
        uint operationVotesCount = votesCountByOperation[operation].add(1);
        votesCountByOperation[operation] = operationVotesCount;

        if (operationVotesCount == 1) {
            allOperationsIndicies[operation] = allOperations.length;
            
            operationsByValidatorIndex[operation] = uint8(ValidatorIndex);
            
            operationsCountByValidatorIndex[uint8(ValidatorIndex)] = uint8(operationsCountByValidatorIndex[uint8(ValidatorIndex)].add(1));
            
            allOperations.push(operation);
            
            
            emit OperationCreated(operation, howMany, validators.length, msg.sender);
        }
        emit OperationUpvoted(operation, operationVotesCount, howMany, validators.length, msg.sender);

        // If enough validators confirmed the same operation
        if (votesCountByOperation[operation] == howMany) {
            deleteOperation(operation);
            emit OperationPerformed(operation, howMany, validators.length, msg.sender);
            return true;
        }

        return false;
    }

    /**
    * @dev Used to delete cancelled or performed operation
    * @param operation defines which operation to delete
    */
    function deleteOperation(bytes32 operation) internal {
        uint index = allOperationsIndicies[operation];
        if (index < allOperations.length - 1) { // Not last
            allOperations[index] = allOperations[allOperations.length.sub(1)];
            allOperationsIndicies[allOperations[index]] = index;
        }
        allOperations.length = allOperations.length.sub(1);

        uint8 ValidatorIndex = uint8(operationsByValidatorIndex[operation]);
        operationsCountByValidatorIndex[ValidatorIndex] = uint8(operationsCountByValidatorIndex[ValidatorIndex].sub(1));

        delete votesMaskByOperation[operation];
        delete votesCountByOperation[operation];
        delete allOperationsIndicies[operation];
        delete operationsByValidatorIndex[operation];
    }

    // PUBLIC METHODS

    /**
    * @dev Allows validators to change their mind by cancelling votesMaskByOperation operations
    * @param operation defines which operation to delete
    */
    function cancelPending(bytes32 operation) public onlyAnyValidator {

        require((isExistValidator(msg.sender) && (validatorsIndices[msg.sender] <= validators.length)), "checkHowManyvalidators: msg.sender is not an Validator");

        uint ValidatorIndex = validatorsIndices[msg.sender].sub(1);
        require((votesMaskByOperation[operation] & (2 ** ValidatorIndex)) != 0, "cancelPending: operation not found for this user");
        votesMaskByOperation[operation] &= ~(2 ** ValidatorIndex);
        uint operationVotesCount = votesCountByOperation[operation].sub(1);
        votesCountByOperation[operation] = operationVotesCount;
        emit OperationDownvoted(operation, operationVotesCount, validators.length, msg.sender);
        if (operationVotesCount == 0) {
            deleteOperation(operation);
            emit OperationCancelled(operation, msg.sender);
        }
    }

    /**
    * @dev Allows validators to change their mind by cancelling all operations
    */

    function cancelAllPending() public onlyManyValidators {
       _cancelAllPending();
    }



    /**Переписать*/

    /**
    * @dev Allows validators to change validatorsship
    * @param newValidators defines array of addresses of new validators
    */
    function transferValidatorShip(address[] memory newValidators) public {
        transferValidatorShipWithHowMany(newValidators, newValidators.length);
    }

    /**
    * @dev Allows validators to change ValidatorShip
    * @param newValidators defines array of addresses of new validators
    * @param newHowManyValidatorsDecide defines how many validators can decide
    */
    function transferValidatorShipWithHowMany(address[] memory newValidators, uint256 newHowManyValidatorsDecide) public onlyManyValidators {
        require(newValidators.length > 0, "transferValidatorShipWithHowMany: validators array is empty");
        require(newValidators.length < 256, "transferValidatorShipWithHowMany: validators count is greater then 255");
        require(newHowManyValidatorsDecide > 0, "transferValidatorShipWithHowMany: newHowManyValidatorsDecide equal to 0");
        require(newHowManyValidatorsDecide <= newValidators.length, "transferValidatorShipWithHowMany: newHowManyValidatorsDecide exceeds the number of Validators");

        // Reset validators reverse lookup table
        for (uint j = 0; j < validators.length; j++) {
            delete validatorsIndices[validators[j]];
        }
        for (uint i = 0; i < newValidators.length; i++) {
            require(newValidators[i] != address(0), "transferValidatorShipWithHowMany: validators array contains zero");
            require(validatorsIndices[newValidators[i]] == 0, "transferValidatorShipWithHowMany: validators array contains duplicates");
            validatorsIndices[newValidators[i]] = uint8(i.add(1));
        }
        
        emit ValidatorShipTransferred(validators, howManyValidatorsDecide, newValidators, newHowManyValidatorsDecide);
        validators = newValidators;
        howManyValidatorsDecide = newHowManyValidatorsDecide;

        _cancelAllPending();
       
        validatorsGeneration++;
    }
}