import { selectionFeature, syncDataLoaderFeature } from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
import { useSuspenseQuery } from '@tanstack/react-query'

import Q from '#/lib/queries'
import type { Entity } from '#/lib/taurpc'
import { cn } from '#/lib/utils'

export default function DatabaseTab() {
	const entitiesQuery = useSuspenseQuery({ ...Q.db.entities })
	const tree = useTree<Entity>({
		rootItemId: 'wb-database-tree-root',
		getItemName: (item) => item.getItemData().name,
		isItemFolder: (item) => item.getItemData().entity_type === 'Schema',
		dataLoader: {
			getItem(itemId) {
				const entities = entitiesQuery.data
				const entity = entities.find((entity) => entity.id === itemId)
				if (!entity) throw new Error('Entity not found')
				return entity
			},
			getChildren(itemId) {
				const entities = entitiesQuery.data
				return entities.filter((entity) => entity.schema_id === itemId).map((e) => e.id)
			},
		},
		features: [syncDataLoaderFeature, selectionFeature],
	})

	console.log(entitiesQuery.data)

	return (
		<div {...tree.getContainerProps()} className="tree">
			{tree.getItems().map((item) => (
				<button
					{...item.getProps()}
					key={item.getId()}
					style={{
						paddingLeft: `${item.getItemMeta().level * 20}px`,
					}}
				>
					<div
						className={cn('treeitem', {
							focused: item.isFocused(),
							expanded: item.isExpanded(),
							selected: item.isSelected(),
							folder: item.isFolder(),
						})}
					>
						{item.getItemName()}
					</div>
				</button>
			))}
		</div>
	)
}
