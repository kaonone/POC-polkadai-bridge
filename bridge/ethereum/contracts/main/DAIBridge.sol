pragma solidity ^0.5.9;

import 'openzeppelin-solidity/contracts/token/ERC20/SafeERC20.sol';

//Beneficieries (validators) template
import "../helpers/ValidatorsOperations.sol";

contract DAIBridge is ValidatorsOperations {

        IERC20 private token;

    enum Status {PENDING,WITHDRAW,APPROVED, CANCELED, CONFIRMED}

        struct Message {
            bytes32 messageID;
            address spender;
            bytes32 substrateAddress;
            uint availableAmount;
            Status status;
        }

        event RelayMessage(bytes32 messageID, address sender, bytes32 recipient, uint amount);
        event RevertMessage(bytes32 messageID, address sender, uint amount);
        event WithdrawMessage(bytes32 MessageID);
        event ApprovedRelayMessage(bytes32 messageID, address  sender,  bytes32 recipient, uint amount);


        mapping(bytes32 => Message) messages;
        mapping(address => Message) messagesBySender;

       /**
       * @notice Constructor.
       * @param _token  Address of DAI token
       */

        constructor (IERC20 _token) public
            ValidatorsOperations() {
            token = _token;
        }  

        // MODIFIERS
        /**
        * @dev Allows to perform method by existing Validator
        */

        modifier onlyExistingValidator(address _Validator) {
            require(isExistValidator(_Validator), "address is not in Validator array");
             _;
        }

        /*
            check available amount
        */

        modifier messageHasAmount(bytes32 messageID) {
            require((messages[messageID].availableAmount > 0), "Amount withdraw");
            _;
        }

        /*
            check that message is valid
        */
        modifier validMessage(bytes32 messageID, address spender, bytes32 substrateAddress, uint availableAmount) {
            require((messages[messageID].spender == spender)
                && (messages[messageID].substrateAddress == substrateAddress)
                && (messages[messageID].availableAmount == availableAmount), "Data is not valid");
            _;
        }

        modifier pendingMessage(bytes32 messageID) {
            require(messages[messageID].status ==  Status.PENDING, "Message is not pending");
            _;
        }

         modifier approvedMessage(bytes32 messageID) {
            require(messages[messageID].status ==  Status.APPROVED, "Message is not approved");
            _;
        }

        function setTransfer(uint amount, bytes32 substrateAddress) public {
            require(token.allowance(msg.sender, address(this)) >= amount, "contract is not allowed to this amount");
            token.transferFrom(msg.sender, address(this), amount);

            bytes32 messageID = keccak256(abi.encodePacked(now));

            Message  memory message = Message(messageID, msg.sender, substrateAddress, amount, Status.PENDING);
            messages[messageID] = message;

            emit RelayMessage(messageID, msg.sender, substrateAddress, amount);
        }

        /*
        * Widthdraw finance by message ID when transfer pending
        */
        function revertTransfer(bytes32 messageID) public pendingMessage(messageID) {
            Message storage message = messages[messageID];

            message.status = Status.CANCELED;

            token.transfer(msg.sender, message.availableAmount);

            emit RevertMessage(messageID, msg.sender, message.availableAmount);
        }


        /*
        * Approve finance by message ID when transfer pending
        */
        function approveTransfer(bytes32 messageID, address spender, bytes32 substrateAddress, uint availableAmount)
            public validMessage(messageID, spender, substrateAddress, availableAmount) pendingMessage(messageID) onlyManyValidators {
            Message storage message = messages[messageID];
            message.status = Status.APPROVED;

            emit ApprovedRelayMessage(messageID, spender, substrateAddress, availableAmount);
        }

        /*
        * Confirm tranfer by message ID when transfer pending
         */
        function confirmTransfer(bytes32 messageID) public approvedMessage(messageID) onlyManyValidators {
            Message storage message = messages[messageID];
            message.status = Status.CONFIRMED;
        }


        /*
        * Withdraw tranfer by message ID after approve from Substrate
        */
        function withdrawTransfer(bytes32 messageID, bytes32  substrateSender, address recipient, uint availableAmount)  public onlyManyValidators {
            require(token.balanceOf(address(this)) >= availableAmount, "Balance is not enough");
            token.transfer(recipient, availableAmount);
            Message  memory message = Message(messageID, msg.sender, substrateSender, availableAmount, Status.WITHDRAW);
            messages[messageID] = message;
            emit WithdrawMessage(messageID);
        }

}