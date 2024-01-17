import {
  NewResource as NewResourceEvent
} from "../generated/EthrDIDLinkedResourcesRegistry/EthrDIDLinkedResourcesRegistry"
import { NewResource as NewResourceEntity } from "../generated/schema"

export function handleNewResource(
  event: NewResourceEvent
): void {
  let entity = new NewResourceEntity(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )

  entity.content = event.params.resource.content

  entity.didIdentity = event.params.didIdentity
  entity.resourceId = event.params.resource.resourceId

  entity.resourceName = event.params.resource.metadata.resourceName
  entity.resourceType = event.params.resource.metadata.resourceType
  entity.resourceVersion = event.params.resource.metadata.resourceVersion
  entity.resourceMediaType = event.params.resource.metadata.mediaType

  entity.metadataChainNodeIndex = event.params.resource.metadata.metadataChainNodeIndex

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}
