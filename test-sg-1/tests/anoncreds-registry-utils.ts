import { newMockEvent } from "matchstick-as"
import { ethereum, Address } from "@graphprotocol/graph-ts"
import {
  NewResourceEvent,
  StatusListUpdateEvent
} from "../generated/AnoncredsRegistry/AnoncredsRegistry"

export function createNewResourceEventEvent(
  didIdentity: Address,
  path: string
): NewResourceEvent {
  let newResourceEventEvent = changetype<NewResourceEvent>(newMockEvent())

  newResourceEventEvent.parameters = new Array()

  newResourceEventEvent.parameters.push(
    new ethereum.EventParam(
      "didIdentity",
      ethereum.Value.fromAddress(didIdentity)
    )
  )
  newResourceEventEvent.parameters.push(
    new ethereum.EventParam("path", ethereum.Value.fromString(path))
  )

  return newResourceEventEvent
}

export function createStatusListUpdateEventEvent(
  indexedRevocationRegistryId: string,
  revocationRegistryId: string,
  statusList: ethereum.Tuple
): StatusListUpdateEvent {
  let statusListUpdateEventEvent = changetype<StatusListUpdateEvent>(
    newMockEvent()
  )

  statusListUpdateEventEvent.parameters = new Array()

  statusListUpdateEventEvent.parameters.push(
    new ethereum.EventParam(
      "indexedRevocationRegistryId",
      ethereum.Value.fromString(indexedRevocationRegistryId)
    )
  )
  statusListUpdateEventEvent.parameters.push(
    new ethereum.EventParam(
      "revocationRegistryId",
      ethereum.Value.fromString(revocationRegistryId)
    )
  )
  statusListUpdateEventEvent.parameters.push(
    new ethereum.EventParam("statusList", ethereum.Value.fromTuple(statusList))
  )

  return statusListUpdateEventEvent
}
