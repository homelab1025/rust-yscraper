import * as React from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import {
    Alert,
    CircularProgress,
    Container,
    Paper,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TablePagination,
    TableRow,
    Typography,
} from '@mui/material';
import { CrateApiCommentsApi, type CommentDto, CommentState } from '../api-client';
import { apiConfig } from '../api-config';
import CommentRow from '../components/CommentRow';

const commentsApi = new CrateApiCommentsApi(apiConfig);
const PAGE_SIZE = 50;
const KEY_NAV_DOWN = 'j';
const KEY_NAV_UP = 'k';
const KEY_PICK = 'p';
const KEY_DISCARD = 'd';

export default function CommentsPage(): React.JSX.Element {
    const [searchParams] = useSearchParams();
    const urlId = searchParams.get('url_id') ? Number(searchParams.get('url_id')) : undefined;
    const filterState = (searchParams.get('state') as CommentState | null) || undefined;

    const [comments, setComments] = useState<CommentDto[]>([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const pendingSelectRef = useRef<'first' | 'last' | null>(null);
    const rowRefs = useRef<(HTMLTableRowElement | null)[]>([]);
    const directionRef = useRef<'down' | 'up'>('down');

    const updateState = useCallback(async (commentId: number, state: CommentState) => {
        try {
            await commentsApi.updateCommentState(commentId, { state });
            // If we are filtering, remove the comment from the list
            if (filterState !== undefined) {
                setComments(prev => prev.filter(c => c.id !== commentId));
                setTotal(t => t - 1);
                // Adjust selection if needed
                if (selectedIndex >= comments.length - 1 && selectedIndex > 0) {
                    setSelectedIndex(i => i - 1);
                }
            } else {
                // Update the state in the local list
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
                const response = await commentsApi.listComments(urlId!, page * PAGE_SIZE, PAGE_SIZE, filterState);
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
    }, [page, urlId, filterState]);

    // After a page change triggered by keyboard nav, snap selection to first or last row.
    useEffect(() => {
        if (loading || pendingSelectRef.current === null) return;
        if (pendingSelectRef.current === 'first') setSelectedIndex(0);
        if (pendingSelectRef.current === 'last') setSelectedIndex(comments.length - 1);
        pendingSelectRef.current = null;
    }, [loading, comments]);

    // Scroll so the selected row (and one below it when going down) stays visible.
    useEffect(() => {
        const scrollTarget =
            directionRef.current === 'down'
                ? (rowRefs.current[selectedIndex + 1] ?? rowRefs.current[selectedIndex])
                : rowRefs.current[selectedIndex];

        if (scrollTarget) {
            // When at the top row and navigating up, scroll the entire page to top
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

    return (
        <Container>
            <Typography variant="h4" gutterBottom>
                Comments
            </Typography>

            {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

            <TableContainer component={Paper}>
                <Table>
                    <TableHead>
                        <TableRow>
                            <TableCell>Comment</TableCell>
                            <TableCell>Author</TableCell>
                            <TableCell>Subcomments</TableCell>
                            <TableCell>Date</TableCell>
                            <TableCell>Actions</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={5} align="center">
                                    <CircularProgress />
                                </TableCell>
                            </TableRow>
                        ) : comments.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={5} align="center">
                                    No comments found.
                                </TableCell>
                            </TableRow>
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
                    </TableBody>
                </Table>
            </TableContainer>

            <TablePagination
                component="div"
                count={total}
                page={page}
                rowsPerPage={PAGE_SIZE}
                rowsPerPageOptions={[PAGE_SIZE]}
                onPageChange={(_e, newPage) => setPage(newPage)}
            />
        </Container>
    );
}
