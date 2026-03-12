import * as React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { type CommentDto, CommentState, SortBy, SortOrder } from '../api-client';
import { useServices } from '../contexts/ServicesContext';
import CommentRow from '../components/CommentRow';

const PAGE_SIZE = 50;
const KEY_NAV_DOWN = 'j';
const KEY_NAV_UP = 'k';
const KEY_PICK = 'p';
const KEY_DISCARD = 'd';

function SortIcon({ active, order }: { active: boolean; order: SortOrder }) {
    if (!active) return <span className="material-symbols-outlined !text-base text-slate-400">unfold_more</span>;
    if (order === SortOrder.Asc) return <span className="material-symbols-outlined !text-base text-primary">expand_less</span>;
    return <span className="material-symbols-outlined !text-base text-primary">expand_more</span>;
}

export default function CommentsPage(): React.JSX.Element {
    const { commentsApi } = useServices();
    const [searchParams] = useSearchParams();
    const urlId = searchParams.get('url_id') ? Number(searchParams.get('url_id')) : undefined;
    const filterState = (searchParams.get('state') as CommentState | null) || undefined;

    const [comments, setComments] = useState<CommentDto[]>([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [sortBy, setSortBy] = useState<SortBy>(SortBy.Date);
    const [sortOrder, setSortOrder] = useState<SortOrder>(SortOrder.Desc);
    const pendingSelectRef = useRef<'first' | 'last' | null>(null);
    const rowRefs = useRef<(HTMLTableRowElement | null)[]>([]);
    const directionRef = useRef<'down' | 'up'>('down');

    const handleRequestSort = (property: SortBy) => {
        if (sortBy === property) {
            setSortOrder(sortOrder === SortOrder.Desc ? SortOrder.Asc : SortOrder.Desc);
        } else {
            setSortBy(property);
            setSortOrder(SortOrder.Desc);
        }
        setPage(0);
        setSelectedIndex(0);
    };

    const updateState = useCallback(async (commentId: number, state: CommentState) => {
        try {
            await commentsApi.updateCommentState(commentId, { state });
            if (filterState !== undefined) {
                setComments(prev => prev.filter(c => c.id !== commentId));
                setTotal(t => t - 1);
                if (selectedIndex >= comments.length - 1 && selectedIndex > 0) {
                    setSelectedIndex(i => i - 1);
                }
            } else {
                setComments(prev => prev.map(c => c.id === commentId ? { ...c, state } : c));
            }
        } catch (err) {
            console.error('Failed to update comment state', err);
        }
    }, [filterState, selectedIndex, comments.length]);

    useEffect(() => {
        const fetchComments = async () => {
            try {
                setLoading(true);
                const response = await commentsApi.listComments(urlId!, page * PAGE_SIZE, PAGE_SIZE, filterState, sortBy, sortOrder);
                setComments(response.data.items);
                setTotal(response.data.total);
                setError(null);
            } catch (err) {
                setError('Failed to fetch comments');
                console.error(err);
            } finally {
                setLoading(false);
            }
        };
        fetchComments();
    }, [page, urlId, filterState, sortBy, sortOrder]);

    useEffect(() => {
        if (loading || pendingSelectRef.current === null) return;
        if (pendingSelectRef.current === 'first') setSelectedIndex(0);
        if (pendingSelectRef.current === 'last') setSelectedIndex(comments.length - 1);
        pendingSelectRef.current = null;
    }, [loading, comments]);

    useEffect(() => {
        const scrollTarget =
            directionRef.current === 'down'
                ? (rowRefs.current[selectedIndex + 1] ?? rowRefs.current[selectedIndex])
                : rowRefs.current[selectedIndex];

        if (scrollTarget) {
            if (selectedIndex === 0 && directionRef.current === 'up') {
                window.scrollTo({ top: 0, behavior: 'smooth' });
            } else {
                scrollTarget.scrollIntoView({ block: 'nearest' });
            }
        }
    }, [selectedIndex]);

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if (loading || comments.length === 0) return;
            const tag = (e.target as HTMLElement).tagName;
            if (tag === 'INPUT' || tag === 'TEXTAREA') return;

            if (e.key === KEY_NAV_DOWN) {
                directionRef.current = 'down';
                if (selectedIndex < comments.length - 1) {
                    setSelectedIndex(i => i + 1);
                } else {
                    const totalPages = Math.ceil(total / PAGE_SIZE);
                    if (page < totalPages - 1) {
                        pendingSelectRef.current = 'first';
                        setPage(p => p + 1);
                    }
                }
            } else if (e.key === KEY_NAV_UP) {
                directionRef.current = 'up';
                if (selectedIndex > 0) {
                    setSelectedIndex(i => i - 1);
                } else if (page > 0) {
                    pendingSelectRef.current = 'last';
                    setPage(p => p - 1);
                }
            } else if (e.key === KEY_PICK) {
                updateState(comments[selectedIndex].id, CommentState.Picked);
            } else if (e.key === KEY_DISCARD) {
                updateState(comments[selectedIndex].id, CommentState.Discarded);
            }
        };

        window.addEventListener('keydown', onKeyDown);
        return () => window.removeEventListener('keydown', onKeyDown);
    }, [loading, comments, selectedIndex, page, total, updateState]);

    const totalPages = Math.ceil(total / PAGE_SIZE);
    const showFrom = total === 0 ? 0 : page * PAGE_SIZE + 1;
    const showTo = Math.min((page + 1) * PAGE_SIZE, total);

    return (
        <div className="max-w-6xl mx-auto flex flex-col gap-6">
            {/* Back link */}
            <Link to="/links" className="flex items-center gap-1 text-sm text-slate-500 hover:text-primary transition-colors w-fit">
                <span className="material-symbols-outlined !text-base">arrow_back</span>
                Links
            </Link>

            {error && (
                <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg text-sm">
                    {error}
                </div>
            )}

            <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
                <div className="px-6 py-5 border-b border-slate-200">
                    <h2 className="text-slate-900 text-xl font-bold tracking-tight">Comments</h2>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-slate-50">
                                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Comment</th>
                                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">Author</th>
                                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">
                                    <button
                                        onClick={() => handleRequestSort(SortBy.SubcommentCount)}
                                        className="flex items-center gap-1 hover:text-slate-900 transition-colors"
                                    >
                                        Subcomments
                                        <SortIcon active={sortBy === SortBy.SubcommentCount} order={sortOrder} />
                                    </button>
                                </th>
                                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500">
                                    <button
                                        onClick={() => handleRequestSort(SortBy.Date)}
                                        className="flex items-center gap-1 hover:text-slate-900 transition-colors"
                                    >
                                        Date
                                        <SortIcon active={sortBy === SortBy.Date} order={sortOrder} />
                                    </button>
                                </th>
                                <th className="px-6 py-4 text-xs font-bold uppercase tracking-wider text-slate-500 text-center">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                            {loading ? (
                                <tr>
                                    <td colSpan={5} className="px-6 py-12 text-center text-slate-500">
                                        <span className="material-symbols-outlined animate-spin !text-3xl">refresh</span>
                                    </td>
                                </tr>
                            ) : comments.length === 0 ? (
                                <tr>
                                    <td colSpan={5} className="px-6 py-12 text-center text-slate-500 text-sm">
                                        No comments found.
                                    </td>
                                </tr>
                            ) : (
                                comments.map((c, i) => (
                                    <CommentRow
                                        key={c.id}
                                        ref={(el) => { rowRefs.current[i] = el; }}
                                        comment={c}
                                        selected={i === selectedIndex}
                                        onUpdateState={updateState}
                                    />
                                ))
                            )}
                        </tbody>
                    </table>
                </div>

                {/* Custom pagination footer */}
                <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex justify-between items-center text-xs text-slate-500">
                    <span>
                        {total > 0 ? `Showing ${showFrom}–${showTo} of ${total} comments` : 'No comments'}
                    </span>
                    <div className="flex gap-4">
                        <button
                            onClick={() => setPage(p => p - 1)}
                            disabled={page === 0}
                            className="hover:text-primary transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                        >
                            Previous
                        </button>
                        <button
                            onClick={() => setPage(p => p + 1)}
                            disabled={page >= totalPages - 1}
                            className="hover:text-primary transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                        >
                            Next
                        </button>
                    </div>
                </div>
            </div>

            {/* Keyboard hint bar */}
            <div className="text-center text-xs text-slate-400 pb-4">
                <kbd className="font-mono">j</kbd>/<kbd className="font-mono">k</kbd> navigate
                {' · '}
                <kbd className="font-mono">p</kbd> pick
                {' · '}
                <kbd className="font-mono">d</kbd> discard
            </div>
        </div>
    );
}
