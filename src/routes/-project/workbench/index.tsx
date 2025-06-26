import React, { Suspense } from 'react'

import { $React } from '@legendapp/state/react-web'

import { DefaultPending } from '#/components/default-screens'
import { cn } from '#/lib/utils'
import WorkbenchStore$ from '#/stores/workbench-store'

import DatabaseTab from './database-tab'
import { WORKBENCH_TABS, type WorkbenchTab } from './misc'

export * from './misc'

export default function ProjectWorkbench() {
	return (
		<div className="flex flex-col h-full gap-1 pt-1">
			<div className="tabs tabs-border tabs-sm w-full overflow-auto justify-center">
				{WORKBENCH_TABS.map((tab) => (
					<React.Fragment key={tab.label}>
						<label className="tab" key={tab.label}>
							<$React.input
								$checked={() => WorkbenchStore$.activeTab.get() === tab.label}
								aria-label={tab.label}
								name="workbench-tab"
								onChange={(e) => {
									if (e.target.checked) WorkbenchStore$.activeTab.set(tab.label)
								}}
								type="radio"
							/>
							<tab.Icon className="size-4 p-0" />
						</label>
					</React.Fragment>
				))}
			</div>

			<TabContentWrapper tab="Database" />
			<TabContentWrapper tab="Queries" />
		</div>
	)
}

function TabContentWrapper({ tab }: { tab: WorkbenchTab }) {
	return (
		<$React.div
			$className={() =>
				cn('overflow-auto flex-1', tab !== WorkbenchStore$.activeTab.get() && 'hidden')
			}
		>
			<Suspense fallback={<DefaultPending showText={false} />}>
				{tab === 'Database' ? <DatabaseTab /> : <QueriesTab />}
			</Suspense>
		</$React.div>
	)
}

function QueriesTab() {
	return <div>Queries Tab</div>
}
