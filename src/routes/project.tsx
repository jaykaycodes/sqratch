import { use$ } from '@legendapp/state/react'
import { createFileRoute } from '@tanstack/react-router'
import { Allotment, LayoutPriority } from 'allotment'

import Q from '#/lib/queries'
import { queryClient } from '#/lib/utils'
import StatusBar from '#/routes/-project/status-bar'
import DetailsPanelStore$ from '#/stores/details-panel-store'
import WorkbenchPanelStore$ from '#/stores/workbench-store'

import ProjectDetails from './-project/details'
import EditorLayout from './-project/editor/editor-layout'
import ProjectWorkbench from './-project/workbench'

export const Route = createFileRoute('/project')({
	loader: async () => {
		queryClient.prefetchQuery({ ...Q.db.entities, staleTime: Infinity, gcTime: Infinity })

		await Promise.all([queryClient.prefetchQuery(Q.project.get)])
	},
	component: ProjectLayout,
})

function ProjectLayout() {
	const isDetailsVisible = use$(DetailsPanelStore$.open)
	const isWorkbenchVisible = use$(WorkbenchPanelStore$.open)

	return (
		<div className="flex size-full flex-col">
			<Allotment proportionalLayout={true}>
				<Allotment.Pane
					key="workbench-panel"
					minSize={170}
					preferredSize={300}
					priority={LayoutPriority.Low}
					visible={isWorkbenchVisible}
				>
					<ProjectWorkbench />
				</Allotment.Pane>

				<Allotment.Pane key="main-panel" minSize={300} priority={LayoutPriority.High}>
					<EditorLayout />
				</Allotment.Pane>

				<Allotment.Pane
					key="details-panel"
					minSize={170}
					preferredSize={300}
					priority={LayoutPriority.Low}
					visible={isDetailsVisible}
				>
					<ProjectDetails />
				</Allotment.Pane>
			</Allotment>

			<StatusBar />
		</div>
	)
}
