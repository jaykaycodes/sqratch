import { Switch, use$ } from '@legendapp/state/react'
import { useQuery } from '@tanstack/react-query'

import Q from '#/lib/queries'
import DetailsPanelStore$ from '#/stores/details-panel-store'

export default function ProjectDetails() {
	return (
		<div className="flex flex-col size-full p-4 overflow-auto">
			<Switch value={DetailsPanelStore$.selectedItems[0].type}>
				{{
					entity: () => <EntityDetails />,
					row: () => <RowDetails />,
					default: () => <BlankDetails />,
				}}
			</Switch>
		</div>
	)
}

function EntityDetails() {
	const selectedItems = use$(DetailsPanelStore$.selectedItems)

	const { data: [entity] = [] } = useQuery({
		...Q.db.entities,
		staleTime: Infinity,
		gcTime: Infinity,
		select: (data) => selectedItems.map((i) => data[i.id]).filter(Boolean),
	})

	if (selectedItems.length > 1) {
		return <div>{selectedItems.length} entities selected</div>
	}

	return (
		<div className="w-full">
			<h1>Entity Details</h1>
			<pre>{JSON.stringify(entity, null, 2)}</pre>
		</div>
	)
}

function RowDetails() {
	return <div>Row</div>
}

function BlankDetails() {
	return <div>Blank</div>
}
