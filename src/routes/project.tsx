import { use$ } from '@legendapp/state/react'
import { createFileRoute, Outlet } from '@tanstack/react-router'
import { Allotment, LayoutPriority } from 'allotment'

import Q from '#/lib/queries'
import { queryClient, taurpc } from '#/lib/utils'
import StatusBar from '#/routes/-project/status-bar'
import uiStore$ from '#/stores/ui-store'

import ProjectDetails from './-project/details'
import ProjectWorkbench from './-project/workbench'

export const Route = createFileRoute('/project')({
	loader: async () => {
		queryClient.prefetchQuery(Q.db.entities)
		return { project: await taurpc.projects.get_project() }
	},
	component: ProjectLayout,
})

function ProjectLayout() {
	const isDetailsVisible = use$(uiStore$.detailsPanel.open)
	const isWorkbenchVisible = use$(uiStore$.workbench.open)

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
					<Outlet />
				</Allotment.Pane>

				<Allotment.Pane
					key="details-panel"
					minSize={170}
					preferredSize={300}
					priority={LayoutPriority.Low}
					snap
					visible={isDetailsVisible}
				>
					<ProjectDetails />
				</Allotment.Pane>
			</Allotment>

			<StatusBar />
		</div>
	)
}
