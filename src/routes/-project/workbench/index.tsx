import { useObservable } from '@legendapp/state/react'

import { useSchemasQuery } from '#/providers/db'
import { favorites$, queries$ } from '#/providers/project'

import WorkbenchSection from './section'
import type { WorkbenchSchemaGroup } from './types'

export default function ProjectWorkbench() {
	const schemas = useSchemasQuery()
	const entities$ = useObservable(
		schemas.data.map(
			(s) =>
				({
					id: s.id,
					name: s.name,
					type: 'Schema',
					items: s.entities.map((e) => ({
						id: e.id,
						name: e.name,
						type: e.entity_type,
					})),
				}) satisfies WorkbenchSchemaGroup,
		),
		[schemas.data],
	)
	console.log('got data', schemas.status, schemas.data)

	return (
		<div className="flex grow flex-col">
			{/* Favorites section */}
			<WorkbenchSection
				className="var(--section-height, 120px)"
				defaultOpen
				emptyText="No favorites yet"
				items$={favorites$}
				title="Favorites"
			/>

			{/* Entities section */}
			<WorkbenchSection
				className="flex-1"
				defaultOpen
				emptyText="No database entities found"
				items$={entities$}
				title="Entities"
			/>

			{/* Queries section */}
			<WorkbenchSection
				className="var(--section-height, 200px)"
				defaultOpen
				emptyText="No queries found"
				items$={queries$}
				title="Queries"
			/>
		</div>
	)
}
