import * as React from 'react';
import { useState } from 'react';
import { useServices } from '../contexts/ServicesContext';

interface AddLinkFormProps {
    onLinkAdded?: () => void;
}

export default function AddLinkForm({ onLinkAdded }: AddLinkFormProps) {
    const { linksApi } = useServices();
    const [itemId, setItemId] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleAddLink = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!itemId) return;
        const id = parseInt(itemId);
        if (isNaN(id) || id <= 0) {
            setError('HN Item ID must be a positive number');
            return;
        }
        try {
            setSubmitting(true);
            setError(null);
            await linksApi.scrapeLink({ item_id: id });
            setItemId('');
            onLinkAdded?.();
        } catch (err: unknown) {
            const apiMsg =
                err &&
                typeof err === 'object' &&
                'response' in err &&
                (err as { response?: { data?: { msg?: string } } }).response?.data?.msg;
            setError(apiMsg || 'Failed to add link');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <form onSubmit={handleAddLink} className="flex items-center gap-2">
            <input
                type="number"
                className="w-40 rounded-lg border border-slate-200 bg-slate-50 focus:ring-2 focus:ring-primary focus:border-primary text-sm px-3 py-1.5 outline-none"
                placeholder="HN Item ID"
                value={itemId}
                onChange={(e) => setItemId(e.target.value)}
                disabled={submitting}
            />
            {error && <span className="text-red-600 text-xs">{error}</span>}
            <button
                type="submit"
                disabled={submitting || !itemId}
                className="bg-primary hover:bg-primary/90 disabled:opacity-50 text-white font-bold py-1.5 px-3 rounded-lg flex items-center gap-1 transition-all text-sm"
            >
                <span className="material-symbols-outlined !text-sm">add</span>
                <span>{submitting ? 'ADDING...' : 'ADD'}</span>
            </button>
        </form>
    );
}
