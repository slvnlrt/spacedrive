import {FolderOpen, Gear} from '@phosphor-icons/react';
import type {SpaceItem as SpaceItemType} from '@sd/ts-client';
import {useNavigate} from 'react-router-dom';
import {
	useContextMenu,
	type ContextMenuItem,
	type ContextMenuResult
} from '../../../hooks/useContextMenu';

interface UseLibraryContextMenuOptions {
	item: SpaceItemType;
}

/**
 * Specialized context menu for Library items in the sidebar or switcher.
 * Includes: Open and Library Settings.
 */
export function useLibraryContextMenu({
	item
}: UseLibraryContextMenuOptions): ContextMenuResult {
	const navigate = useNavigate();

	const items: ContextMenuItem[] = [
		{
			icon: FolderOpen,
			label: 'Open Library',
			onClick: () => {
				// Navigate to library root
				navigate(`/explorer`);
			}
		},
		{type: 'separator'},
		{
			icon: Gear,
			label: 'Library Settings',
			onClick: () => {
				navigate('/settings');
			}
		}
	];

	return useContextMenu({items});
}
