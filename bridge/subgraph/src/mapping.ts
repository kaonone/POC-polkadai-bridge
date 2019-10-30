import { BigInt } from "@graphprotocol/graph-ts"
import {
  Contract,
  BridgeStoppedMessage,
  BridgeStartedMessage,
  BridgePausedMessage,
  BridgeResumedMessage,
  RelayMessage,
  RevertMessage,
  WithdrawMessage,
  ApprovedRelayMessage,
  ConfirmeMessage,
  WithdrawTransferCall,
} from "../generated/Contract/Contract"
import { Message, Entry } from "../generated/schema"

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

export function handleConfirmMessage(event: ConfirmeMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED")
}

export function handleBridgeStartedMessage(event: BridgeStartedMessage): void {
  let message = new Entry(event.params.messageID.toHex())
  message.ethAddress = event.params.sender.toHexString()
  message.status = "PENDING"
  message.action = "START"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleBridgeStoppedMessage(event: BridgeStoppedMessage): void {
  let message = new Entry(event.params.messageID.toHex())
  message.ethAddress = event.params.sender.toHexString()
  message.status = "PENDING"
  message.action = "STOP"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleBridgePausedMessage(event: BridgePausedMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED")
}

export function handleBridgeResumedMessage(event: BridgeResumedMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED")
}

function changeMessageStatus(id: String, status: String): void {
  let message = Message.load(id)
  if (message != null) {
    message.status = status
    message.save()
  }
}
