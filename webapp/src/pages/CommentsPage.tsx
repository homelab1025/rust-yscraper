import * as React from 'react';
import { useEffect, useState } from 'react';
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
import { CrateApiCommentsApi, type CommentDto } from '../api-client';
import { apiConfig } from '../api-config';
import CommentRow from '../components/CommentRow';

const commentsApi = new CrateApiCommentsApi(apiConfig);
const PAGE_SIZE = 50;

export default function CommentsPage(): React.JSX.Element {
    const [searchParams] = useSearchParams();
    const urlId = searchParams.get('url_id') ? Number(searchParams.get('url_id')) : undefined;

    const [comments, setComments] = useState<CommentDto[]>([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchComments = async () => {
            try {
                setLoading(true);
                const response = await commentsApi.listComments(page * PAGE_SIZE, PAGE_SIZE, urlId);
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
    }, [page, urlId]);

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
                            <TableCell>Date</TableCell>
                            <TableCell>Actions</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={4} align="center">
                                    <CircularProgress />
                                </TableCell>
                            </TableRow>
                        ) : comments.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} align="center">
                                    No comments found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            comments.map((c) => <CommentRow key={c.id} comment={c} />)
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
