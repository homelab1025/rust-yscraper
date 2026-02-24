import * as React from 'react';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import {
    Alert,
    Button,
    CircularProgress,
    Container,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    IconButton,
    Paper,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    Typography
} from '@mui/material';
import DeleteIcon from '@mui/icons-material/Delete';
import CommentIcon from '@mui/icons-material/Comment';
import { AddLink } from "../components/AddLink.tsx";
import { CrateApiLinksApi, type LinkDto } from "../api-client";
import { apiConfig } from "../api-config";

const linksApi = new CrateApiLinksApi(apiConfig);

export default function LinkManagementPage(): React.JSX.Element {
    const [links, setLinks] = useState<LinkDto[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
    const [linkToDelete, setLinkToDelete] = useState<number | null>(null);

    const fetchLinks = async () => {
        try {
            setLoading(true);
            const response = await linksApi.listLinks();
            setLinks(response.data);
            setError(null);
        } catch (err) {
            setError('Failed to fetch links');
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const handleDelete = (id: number) => {
        setLinkToDelete(id);
        setDeleteDialogOpen(true);
    };

    const handleConfirmDelete = async () => {
        if (linkToDelete === null) return;

        try {
            await linksApi.deleteLink(linkToDelete);
            await fetchLinks();
        } catch (err) {
            setError('Failed to delete link');
            console.error(err);
        } finally {
            setDeleteDialogOpen(false);
            setLinkToDelete(null);
        }
    };

    const handleCloseDeleteDialog = () => {
        setDeleteDialogOpen(false);
        setLinkToDelete(null);
    };

    useEffect(() => {
        fetchLinks();
    }, []);

    return (
        <Container>
            <Typography variant="h4" gutterBottom>
                Link Management
            </Typography>

            <AddLink onSuccess={fetchLinks} />

            {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

            <TableContainer component={Paper}>
                <Table>
                    <TableHead>
                        <TableRow>
                            <TableCell>ID</TableCell>
                            <TableCell>URL</TableCell>
                            <TableCell>Date Added</TableCell>
                            <TableCell>Comments</TableCell>
                            <TableCell>Status</TableCell>
                            <TableCell>Actions</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={6} align="center">
                                    <CircularProgress />
                                </TableCell>
                            </TableRow>
                        ) : links.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} align="center">
                                    No links found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            links.map((link) => (
                                <TableRow key={link.id}>
                                    <TableCell>{link.id}</TableCell>
                                    <TableCell>
                                        <a href={link.url} target="_blank" rel="noopener noreferrer">
                                            {link.url}
                                        </a>
                                    </TableCell>
                                    <TableCell>{new Date(link.date_added).toLocaleString()}</TableCell>
                                    <TableCell>{link.comment_count}</TableCell>
                                    <TableCell>{'Scraped'}</TableCell>
                                    <TableCell>
                                        <IconButton
                                            component={Link}
                                            to={`/comments?url_id=${link.id}`}
                                            aria-label="see comments"
                                            color="primary"
                                        >
                                            <CommentIcon />
                                        </IconButton>
                                        <IconButton
                                            aria-label="delete"
                                            color="error"
                                            onClick={() => handleDelete(link.id)}
                                        >
                                            <DeleteIcon />
                                        </IconButton>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </TableContainer>

            <Dialog
                open={deleteDialogOpen}
                onClose={handleCloseDeleteDialog}
                aria-labelledby="delete-dialog-title"
                aria-describedby="delete-dialog-description"
            >
                <DialogTitle id="delete-dialog-title">
                    {"Confirm Delete"}
                </DialogTitle>
                <DialogContent>
                    <DialogContentText id="delete-dialog-description">
                        Are you sure you want to delete this link? This action cannot be undone.
                    </DialogContentText>
                </DialogContent>
                <DialogActions>
                    <Button onClick={handleCloseDeleteDialog}>Cancel</Button>
                    <Button onClick={handleConfirmDelete} color="error" autoFocus>
                        Delete
                    </Button>
                </DialogActions>
            </Dialog>
        </Container>
    );
}