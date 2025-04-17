import { Memo } from '@legendapp/state/react'

import { ResizableHandle, ResizablePanelGroup } from '#/components/ui/resizable'
import { useProjectStore$ } from '#/providers/project'

import WorkbenchSection from './section'

export * from './types'

export default function ProjectWorkbench() {
	const project$ = useProjectStore$()

	return (
		<ResizablePanelGroup className="h-fit flex-1" direction="vertical">
			{/* Favorites */}
			<WorkbenchSection
				emptyText="No favorites yet"
				state$={project$.workbench.favorites}
				title="Favorites"
			/>

			<Memo>{() => <ResizableHandle disabled={!project$.workbench.favorites.open.get()} />}</Memo>

			{/* Entities */}
			<WorkbenchSection
				emptyText="No database entities found"
				state$={project$.workbench.entities}
				title="Entities"
			/>

			<Memo>{() => <ResizableHandle disabled={!project$.workbench.entities.open.get()} />}</Memo>

			{/* Queries */}
			<WorkbenchSection
				emptyText="No queries found"
				state$={project$.workbench.queries}
				title="Queries"
			/>
		</ResizablePanelGroup>
	)
}
