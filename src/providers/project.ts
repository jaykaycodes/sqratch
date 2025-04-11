import { observable } from '@legendapp/state'

import type { WorkbenchItemOrGroup } from '#/routes/-project/workbench/types'

export const queries$ = observable([
	{
		id: 'dir1',
		type: 'Folder',
		name: 'Reports',
		items: [
			{ id: 'q1', type: 'UserQuery', name: 'Monthly active users', favorited: true },
			{ id: 'q2', type: 'UserQuery', name: 'Revenue report', favorited: true },
			{
				id: 'subdir1',
				type: 'Folder',
				name: 'Marketing',
				items: [{ id: 'q3', type: 'UserQuery', name: 'Campaign performance', favorited: false }],
			},
		],
	},
	{
		id: 'dir2',
		type: 'Folder',
		name: 'Analytics',
		items: [
			{ id: 'q4', type: 'UserQuery', name: 'User retention', favorited: false },
			{ id: 'q5', type: 'UserQuery', name: 'Funnel analysis', favorited: false },
		],
	},
] as WorkbenchItemOrGroup[])

export const favorites$ = observable([
	{ id: 'fav1', type: 'UserQuery', name: 'Monthly active users', favorited: true },
	{ id: 'fav2', type: 'Table', name: 'users', favorited: true },
	{ id: 'fav3', type: 'UserQuery', name: 'Revenue report', favorited: true },
] as WorkbenchItemOrGroup[])
