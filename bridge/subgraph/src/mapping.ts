import { BigInt } from "@graphprotocol/graph-ts"
import {
  Contract,
  RelayMessage,
  RevertMessage,
  WithdrawMessage,
  ApprovedRelayMessage,
  WithdrawTransferCall,
} from "../generated/Contract/Contract"
import { Message } from "../generated/schema"

export function handleRelayMessage(event: RelayMessage): void {
  let message = new Message(event.params.messageID.toHex())
  message.ethAddress = event.params.sender.toHexString()
  message.subAddress = event.params.recipient.toHexString()
  message.amount = event.params.amount
  message.status = "PENDING"
  message.direction = "ETH2SUB"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleRevertMessage(event: RevertMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED")
}

export function handleWithdrawMessage(event: WithdrawMessage): void {
  let message = new Message(event.params.MessageID.toHex())
  message.ethAddress = event.params.substrateSender.toHexString()
  message.subAddress = event.params.recipient.toHexString()
  message.amount = event.params.amount
  message.status = "WITHDRAW"
  message.direction = "SUB2ETH"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleApprovedRelayMessage(event: ApprovedRelayMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "APPROVED")
}

function changeMessageStatus(id: String, status: String): void {
  let message = Message.load(id)
  if (message != null) {
    message.status = status
    message.save()
  }
}
