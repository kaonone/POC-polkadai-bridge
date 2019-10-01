// @flow
import EVMRevert from './helpers/EVMRevert';

require('chai')
    .use(require('chai-as-promised'))
    .use(require('chai-bignumber')(web3.BigNumber))
    .should();

const ValidatorOperations = artifacts.require('ValidatorsOperations.sol');
const ValidatorOperationsImpl = artifacts.require('ValidatorOperationsImpl.sol');

contract('ValidatorOperations', function ([_, wallet1, wallet2, wallet3, wallet4, wallet5]) {
    it('should be initialized correctly', async function () {
        const obj = await ValidatorOperations.new();

        (await obj.validators.call(0)).should.be.equal(_);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(1);

        (await obj.isExistValidator.call(_)).should.be.true;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.false;
        (await obj.isExistValidator.call(wallet4)).should.be.false;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 1 => 1 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1]);

        (await obj.validators.call(0)).should.be.equal(wallet1);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(1);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.true;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.false;
        (await obj.isExistValidator.call(wallet4)).should.be.false;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 1 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2]);

        (await obj.validators.call(0)).should.be.equal(wallet1);
        (await obj.validators.call(1)).should.be.equal(wallet2);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.true;
        (await obj.isExistValidator.call(wallet2)).should.be.true;
        (await obj.isExistValidator.call(wallet3)).should.be.false;
        (await obj.isExistValidator.call(wallet4)).should.be.false;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 1 => 3 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);

        (await obj.validators.call(0)).should.be.equal(wallet1);
        (await obj.validators.call(1)).should.be.equal(wallet2);
        (await obj.validators.call(2)).should.be.equal(wallet3);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(3);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.true;
        (await obj.isExistValidator.call(wallet2)).should.be.true;
        (await obj.isExistValidator.call(wallet3)).should.be.true;
        (await obj.isExistValidator.call(wallet4)).should.be.false;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 2 => 1 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2]);
        
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);

        await obj.transferValidatorShip([wallet3], { from: wallet1 });
        await obj.transferValidatorShip([wallet3], { from: wallet2 });
        
        (await obj.validators.call(0)).should.be.equal(wallet3);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(1);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.true;
        (await obj.isExistValidator.call(wallet4)).should.be.false;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 3 => 1 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);
        await obj.transferValidatorShip([wallet4], { from: wallet1 });
        await obj.transferValidatorShip([wallet4], { from: wallet2 });
        await obj.transferValidatorShip([wallet4], { from: wallet3 });

        (await obj.validators.call(0)).should.be.equal(wallet4);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(1);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.false;
        (await obj.isExistValidator.call(wallet4)).should.be.true;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 2 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2]);
        await obj.transferValidatorShip([wallet3, wallet4], { from: wallet1 });
        await obj.transferValidatorShip([wallet3, wallet4], { from: wallet2 });

        (await obj.validators.call(0)).should.be.equal(wallet3);
        (await obj.validators.call(1)).should.be.equal(wallet4);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.true;
        (await obj.isExistValidator.call(wallet4)).should.be.true;
        (await obj.isExistValidator.call(wallet5)).should.be.false;
    });

    it('should transfer validatorship 2 => 3 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2]);
        await obj.transferValidatorShip([wallet3, wallet4, wallet5], { from: wallet1 });
        await obj.transferValidatorShip([wallet3, wallet4, wallet5], { from: wallet2 });

        (await obj.validators.call(0)).should.be.equal(wallet3);
        (await obj.validators.call(1)).should.be.equal(wallet4);
        (await obj.validators.call(2)).should.be.equal(wallet5);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(3);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.true;
        (await obj.isExistValidator.call(wallet4)).should.be.true;
        (await obj.isExistValidator.call(wallet5)).should.be.true;
    });

    it('should transfer validatorship 3 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet1 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet2 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet3 });

        (await obj.validators.call(0)).should.be.equal(wallet4);
        (await obj.validators.call(1)).should.be.equal(wallet5);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);

        (await obj.isExistValidator.call(_)).should.be.false;
        (await obj.isExistValidator.call(wallet1)).should.be.false;
        (await obj.isExistValidator.call(wallet2)).should.be.false;
        (await obj.isExistValidator.call(wallet3)).should.be.false;
        (await obj.isExistValidator.call(wallet4)).should.be.true;
        (await obj.isExistValidator.call(wallet5)).should.be.true;
    });

    it('should transfer validatorship 1,2 of 3 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShipWithHowMany([wallet1, wallet2, wallet3], 2);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet1 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet2 });

        (await obj.validators.call(0)).should.be.equal(wallet4);
        (await obj.validators.call(1)).should.be.equal(wallet5);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);
    });

    it('should transfer validatorship 2,3 of 3 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShipWithHowMany([wallet1, wallet2, wallet3], 2);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet2 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet3 });

        (await obj.validators.call(0)).should.be.equal(wallet4);
        (await obj.validators.call(1)).should.be.equal(wallet5);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);
    });


    it('should transfer validatorship 5 => 3 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShipWithHowMany([wallet1, wallet2, wallet3, wallet4, wallet5], 3);
        

        (await obj.validators.call(0)).should.be.equal(wallet1);
        (await obj.validators.call(1)).should.be.equal(wallet2);


        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet1 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet2 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet3 });
        
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);
    });

    it('should transfer validatorship 1,3 of 3 => 2 correctly', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShipWithHowMany([wallet1, wallet2, wallet3], 2);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet1 });
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet3 });

        (await obj.validators.call(0)).should.be.equal(wallet4);
        (await obj.validators.call(1)).should.be.equal(wallet5);
        (await obj.validatorsCount.call()).toNumber().should.be.equal(2);
    });

    it('should not transfer validatorship with wrong how many argument', async function () {
        const obj = await ValidatorOperations.new();

        await obj.transferValidatorShipWithHowMany([wallet1], 0).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShipWithHowMany([wallet1, wallet2], 3).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShipWithHowMany([wallet1, wallet2], 4).should.be.rejectedWith(EVMRevert);
    });

    it('should correctly manage allOperations array', async function () {
        const obj = await ValidatorOperations.new();

        // Transfer validatorship 1 => 1
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
        await obj.transferValidatorShip([wallet1]);
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);

        // Transfer validatorship 1 => 2
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
        await obj.transferValidatorShip([wallet2, wallet3], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);

        // Transfer validatorship 2 => 2
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);
        await obj.transferValidatorShip([wallet4, wallet5], { from: wallet3 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
    });

    it('should allow to cancel pending operations', async function () {
        const obj = await ValidatorOperations.new();
        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);

        // First owner agree
        await obj.transferValidatorShip([wallet4], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // First owner disagree
        const operation1 = await obj.allOperations.call(0);
        await obj.cancelPending(operation1, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);

        // First and Second validators agree
        await obj.transferValidatorShip([wallet4], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);
        await obj.transferValidatorShip([wallet4], { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // Second owner disagree
        const operation2 = await obj.allOperations.call(0);
        await obj.cancelPending(operation2, { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // Third owner agree
        await obj.transferValidatorShip([wallet4], { from: wallet3 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // Second owner agree
        await obj.transferValidatorShip([wallet4], { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
    });

    it('should reset all pending operations when validators change', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.setValue(1, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        await obj.transferValidatorShip([wallet3], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(2);

        await obj.transferValidatorShip([wallet3], { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);
    });

    it('should correctly perform last operation', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.setValue(1, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        await obj.transferValidatorShip([wallet3], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(2);

        await obj.transferValidatorShip([wallet3], { from: wallet2 });
        (await obj.validators.call(0)).should.be.equal(wallet3);
    });

    it('should correctly perform not last operation', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.setValue(1, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        await obj.transferValidatorShip([wallet3], { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(2);

        await obj.setValue(1, { from: wallet2 });
        (await obj.value.call()).toNumber().should.be.equal(1);
    });

    it('should handle multiple simultaneous operations correctly', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        // wallet1 => 1
        await obj.setValue(1, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // Check value
        (await obj.value.call()).toNumber().should.be.equal(0);

        // wallet2 => 2
        await obj.setValue(2, { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(2);

        // Check value
        (await obj.value.call()).toNumber().should.be.equal(0);

        // wallet1 => 2
        await obj.setValue(2, { from: wallet1 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(1);

        // Check value
        (await obj.value.call()).toNumber().should.be.equal(2);

        // wallet2 => 1
        await obj.setValue(1, { from: wallet2 });
        (await obj.allOperationsCount.call()).toNumber().should.be.equal(0);

        // Check value
        (await obj.value.call()).toNumber().should.be.equal(1);
    });

    it('should allow to call onlyAnyValidator methods properly', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        // Not validators try to call
        await obj.setValueAny(1, { from: _ }).should.be.rejectedWith(EVMRevert);
        await obj.setValueAny(1, { from: wallet3 }).should.be.rejectedWith(EVMRevert);

        // validators try to call
        await obj.setValueAny(2, { from: wallet1 }).should.be.fulfilled;
        (await obj.value.call()).toNumber().should.be.equal(2);
        await obj.setValueAny(3, { from: wallet2 }).should.be.fulfilled;
        (await obj.value.call()).toNumber().should.be.equal(3);
    });

    it('should allow to call onlyManyvalidators methods properly', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        // Not validators try to call
        await obj.setValue(1, { from: _ }).should.be.rejectedWith(EVMRevert);
        await obj.setValue(1, { from: wallet3 }).should.be.rejectedWith(EVMRevert);

        // Single validators try to call twice
        await obj.setValue(2, { from: wallet1 }).should.be.fulfilled;
        await obj.setValue(2, { from: wallet1 }).should.be.rejectedWith(EVMRevert);
    });

    it('should allow to call onlyAllvalidators methods properly', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShipWithHowMany([wallet1, wallet2], 1);

        // Not validators try to call
        await obj.setValueAll(1, { from: _ }).should.be.rejectedWith(EVMRevert);
        await obj.setValueAll(1, { from: wallet3 }).should.be.rejectedWith(EVMRevert);

        // Single validators try to call twice
        await obj.setValueAll(2, { from: wallet1 }).should.be.fulfilled;
        await obj.setValueAll(2, { from: wallet2 }).should.be.fulfilled;
        (await obj.value.call()).toNumber().should.be.equal(2);
    });

    it('should allow to call onlySomevalidators(n) methods properly', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        // Invalid arg
        await obj.setValueSome(1, 0, { from: _ }).should.be.rejectedWith(EVMRevert);
        await obj.setValueSome(1, 3, { from: _ }).should.be.rejectedWith(EVMRevert);

        // Not validators try to call
        await obj.setValueSome(1, 1, { from: _ }).should.be.rejectedWith(EVMRevert);
        await obj.setValueSome(1, 1, { from: wallet3 }).should.be.rejectedWith(EVMRevert);

        // validators try to call
        await obj.setValueSome(5, 2, { from: wallet1 }).should.be.fulfilled;
        (await obj.value.call()).toNumber().should.be.equal(0);
        await obj.setValueSome(5, 2, { from: wallet2 }).should.be.fulfilled;
        (await obj.value.call()).toNumber().should.be.equal(5);
    });

    it('should not allow to cancel pending of another owner', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        // First owner
        await obj.setValue(2, { from: wallet1 }).should.be.fulfilled;

        // Second owner
        const operation = await obj.allOperations.call(0);
        await obj.cancelPending(operation, { from: wallet2 }).should.be.rejectedWith(EVMRevert);
    });

    it('should not allow to transfer validatorship to no one and to user 0', async function () {

        const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";
        const obj = await ValidatorOperations.new();
        await obj.transferValidatorShip([]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([ZERO_ADDRESS]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([ZERO_ADDRESS, wallet1]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([wallet1, ZERO_ADDRESS]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([ZERO_ADDRESS, wallet1, wallet2]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([wallet1, ZERO_ADDRESS, wallet2]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([wallet1, wallet2, ZERO_ADDRESS]).should.be.rejectedWith(EVMRevert);
    });

    it('should works for nested methods with onlyManyvalidators modifier', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.nestedFirst(100, { from: wallet1 });
        await obj.nestedFirst(100, { from: wallet2 });

        (await obj.value.call()).toNumber().should.be.equal(100);
    });

    it('should works for nested methods with onlyAnyvalidators modifier', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.nestedFirstAnyToAny(100, { from: wallet3 }).should.be.rejectedWith(EVMRevert);
        await obj.nestedFirstAnyToAny2(100, { from: wallet1 }).should.be.rejectedWith(EVMRevert);

        await obj.nestedFirstAnyToAny(100, { from: wallet1 });
        await obj.nestedFirstAnyToAny(100, { from: wallet2 });
        (await obj.value.call()).toNumber().should.be.equal(100);
    });

    it('should works for nested methods with onlyAllvalidators modifier', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2]);

        await obj.nestedFirstAllToAll(100, { from: wallet3 }).should.be.rejectedWith(EVMRevert);
        await obj.nestedFirstAllToAll2(100, { from: wallet1 }).should.be.fulfilled;
        await obj.nestedFirstAllToAll2(100, { from: wallet2 }).should.be.rejectedWith(EVMRevert);

        await obj.nestedFirstAllToAll(100, { from: wallet1 });
        await obj.nestedFirstAllToAll(100, { from: wallet2 });
        (await obj.value.call()).toNumber().should.be.equal(100);
    });

    it('should works for nested methods with onlyManyvalidators => onlySomevalidators modifier', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);

        await obj.nestedFirstManyToSome(100, 1, { from: wallet1 });
        await obj.nestedFirstManyToSome(100, 1, { from: wallet2 });
        await obj.nestedFirstManyToSome(100, 1, { from: wallet3 });
        (await obj.value.call()).toNumber().should.be.equal(100);

        await obj.nestedFirstManyToSome(200, 2, { from: wallet1 });
        await obj.nestedFirstManyToSome(200, 2, { from: wallet2 });
        await obj.nestedFirstManyToSome(200, 2, { from: wallet3 });
        (await obj.value.call()).toNumber().should.be.equal(200);

        await obj.nestedFirstManyToSome(300, 3, { from: wallet1 });
        await obj.nestedFirstManyToSome(300, 3, { from: wallet2 });
        await obj.nestedFirstManyToSome(300, 3, { from: wallet3 });
        (await obj.value.call()).toNumber().should.be.equal(300);
    });

    it('should works for nested methods with onlyAnyvalidators => onlySomevalidators modifier', async function () {
        const obj = await ValidatorOperationsImpl.new();
        await obj.transferValidatorShip([wallet1, wallet2, wallet3]);

        // 1 => 1
        await obj.nestedFirstAnyToSome(100, 1, { from: wallet1 });
        (await obj.value.call()).toNumber().should.be.equal(100);
        await obj.nestedFirstAnyToSome(200, 1, { from: wallet2 });
        (await obj.value.call()).toNumber().should.be.equal(200);
        await obj.nestedFirstAnyToSome(300, 1, { from: wallet3 });
        (await obj.value.call()).toNumber().should.be.equal(300);

        // 1 => 2
        await obj.nestedFirstAnyToSome(100, 2, { from: wallet1 }).should.be.rejectedWith(EVMRevert);
        await obj.nestedFirstAnyToSome(200, 2, { from: wallet2 }).should.be.rejectedWith(EVMRevert);
        await obj.nestedFirstAnyToSome(300, 2, { from: wallet3 }).should.be.rejectedWith(EVMRevert);
    });

    it('should not allow to transfer validatorship to several equal users', async function () {
        const obj = await ValidatorOperations.new();
        await obj.transferValidatorShip([wallet1, wallet1]).should.be.rejectedWith(EVMRevert);
        await obj.transferValidatorShip([wallet1, wallet2, wallet1]).should.be.rejectedWith(EVMRevert);
    });

    it('should not allow to transfer validatorship to more than 256 validators', async function () {
        const obj = await ValidatorOperations.new();
        await obj.transferValidatorShip([
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _,
            _,
        ]).should.be.rejectedWith(EVMRevert);
    });
});
