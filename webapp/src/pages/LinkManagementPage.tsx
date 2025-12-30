import * as React from 'react';
import {useEffect, useState} from 'react';
import {
    Alert,
    CircularProgress,
    Container,
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
import {AddLink} from "../components/AddLink.tsx";
import {CrateApiLinksApi, type LinkDto} from "../api-client";

const linksApi = new CrateApiLinksApi();

export default function LinkManagementPage(): React.JSX.Element {
    const [links, setLinks] = useState<LinkDto[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

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

    const handleDelete = async (id: number) => {
        if (!window.confirm('Are you sure you want to delete this link?')) {
            return;
        }

        try {
            await linksApi.deleteLink(id);
            await fetchLinks();
        } catch (err) {
            setError('Failed to delete link');
            console.error(err);
        }
    };

    useEffect(() => {
        fetchLinks();
    }, []);

    return (
        <Container>
            <Typography variant="h4" gutterBottom>
                Link Management
            </Typography>

            <AddLink/>

            {error && <Alert severity="error" sx={{mb: 2}}>{error}</Alert>}

            <TableContainer component={Paper}>
                <Table>
                    <TableHead>
                        <TableRow>
                            <TableCell>ID</TableCell>
                            <TableCell>URL</TableCell>
                            <TableCell>Date Added</TableCell>
                            <TableCell>Status</TableCell>
                            <TableCell>Actions</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={5} align="center">
                                    <CircularProgress/>
                                </TableCell>
                            </TableRow>
                        ) : links.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={5} align="center">
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
                                    <TableCell>{'Scraped'}</TableCell>
                                    <TableCell>
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
        </Container>
    );
}