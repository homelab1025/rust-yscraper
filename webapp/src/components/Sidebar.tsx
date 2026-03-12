import * as React from 'react';
import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useServices } from '../contexts/ServicesContext';

interface SidebarProps {
    onLinkAdded?: () => void;
}

export default function Sidebar({ onLinkAdded }: SidebarProps) {
    const { linksApi } = useServices();
    const location = useLocation();
    const [itemId, setItemId] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const isActive = (path: string) => location.pathname === path;

    const handleAddLink = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!itemId) return;
        try {
            setSubmitting(true);
            setError(null);
            await linksApi.scrapeLink({ item_id: parseInt(itemId) });
            setItemId('');
            onLinkAdded?.();
        } catch {
            setError('Failed to add link');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <aside className="w-80 border-r border-slate-200 bg-white p-6 flex flex-col gap-6 overflow-y-auto">
            <div className="flex flex-col gap-2 border-b border-slate-100 pb-6">
                <Link
                    to="/links"
                    className={`text-sm font-semibold transition-colors flex items-center gap-3 px-2 py-1.5 rounded-lg hover:bg-slate-50 ${
                        isActive('/links')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg">link</span>
                    LINKS
                </Link>
                <Link
                    to="/about"
                    className={`text-sm font-semibold transition-colors flex items-center gap-3 px-2 py-1.5 rounded-lg hover:bg-slate-50 ${
                        isActive('/about')
                            ? 'text-primary bg-slate-50'
                            : 'text-slate-600 hover:text-primary'
                    }`}
                >
                    <span className="material-symbols-outlined !text-lg">info</span>
                    ABOUT
                </Link>
            </div>

            <div>
                <h3 className="text-slate-900 text-sm font-bold uppercase tracking-wider mb-4">Add New Link</h3>
                <form onSubmit={handleAddLink} className="flex flex-col gap-4">
                    <label className="flex flex-col gap-2">
                        <span className="text-slate-600 text-xs font-medium uppercase">Item ID</span>
                        <input
                            type="number"
                            className="w-full rounded-lg border border-slate-200 bg-slate-50 focus:ring-2 focus:ring-primary focus:border-primary text-sm p-3 outline-none"
                            placeholder="Enter HN Item ID"
                            value={itemId}
                            onChange={(e) => setItemId(e.target.value)}
                            disabled={submitting}
                        />
                    </label>
                    {error && <p className="text-red-600 text-xs">{error}</p>}
                    <button
                        type="submit"
                        disabled={submitting || !itemId}
                        className="w-full bg-primary hover:bg-primary/90 disabled:opacity-50 text-white font-bold py-3 px-4 rounded-lg flex items-center justify-center gap-2 transition-all"
                    >
                        <span className="material-symbols-outlined !text-sm">add</span>
                        <span>{submitting ? 'ADDING...' : 'ADD'}</span>
                    </button>
                </form>
            </div>
        </aside>
    );
}
