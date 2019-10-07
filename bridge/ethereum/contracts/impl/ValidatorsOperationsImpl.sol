pragma solidity ^0.5.0;

import "../helpers/ValidatorsOperations.sol";


contract ValidatorOperationsImpl is ValidatorsOperations {

    uint public value;

    function setValue(uint _value) public onlyManyValidators {
        value = _value;
    }

    function setValueAny(uint _value) public onlyAnyValidator {
        value = _value;
    }

    function setValueAll(uint _value) public onlyAllValidators {
        value = _value;
    }

    function setValueSome(uint _value, uint howMany) public onlySomeValidators(howMany) {
        value = _value;
    }

    function nestedFirst(uint _value) public onlyManyValidators {
        nestedSecond(_value);
    }

    function nestedSecond(uint _value) public onlyManyValidators {
        value = _value;
    }

    //

    function nestedFirstAllToAll(uint _value) public onlyAllValidators {
        nestedSecondAllToAll(_value);
    }

    function nestedFirstAllToAll2(uint _value) public onlyAllValidators {
        this.nestedSecondAllToAll(_value); // this.
    }

    function nestedSecondAllToAll(uint _value) public onlyAllValidators {
        value = _value;
    }

    //

    function nestedFirstAnyToAny(uint _value) public onlyAnyValidator {
        nestedSecondAnyToAny(_value);
    }

    function nestedFirstAnyToAny2(uint _value) public onlyAnyValidator {
        this.nestedSecondAnyToAny(_value); // this.
    }

    function nestedSecondAnyToAny(uint _value) public onlyAnyValidator {
        value = _value;
    }

    //

    function nestedFirstManyToSome(uint _value, uint howMany) public onlyManyValidators {
        nestedSecondSome(_value, howMany);
    }

    function nestedFirstAnyToSome(uint _value, uint howMany) public onlyAnyValidator {
        nestedSecondSome(_value, howMany);
    }

    function nestedSecondSome(uint _value, uint howMany) public onlySomeValidators(howMany) {
        value = _value;
    }

}