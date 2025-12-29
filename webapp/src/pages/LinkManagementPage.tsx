import * as React from 'react';
import {useEffect, useState} from 'react';
import {
    Alert,
    Box,
    Button,
    CircularProgress,
    Container,
    Paper,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    TextField,
    Typography
} from '@mui/material';
import axios from 'axios';

interface LinkDto {
    id: number;
    url: string;
    date_added: string;
    status?: string; // Adding status as mentioned in requirements, although not yet in backend
}

export default function LinkManagementPage(): React.JSX.Element {
    const [links, setLinks] = useState<LinkDto[]>([]);
    const [newLinkId, setNewLinkId] = useState('');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);

    const fetchLinks = async () => {
        try {
            setLoading(true);
            const response = await axios.get<LinkDto[]>('/api/links');
            setLinks(response.data);
            setError(null);
        } catch (err) {
            setError('Failed to fetch links');
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchLinks();
    }, []);

    const handleAddLink = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newLinkId) return;

        try {
            setSubmitting(true);
            await axios.post('/api/scrape', {item_id: parseInt(newLinkId)});
            setNewLinkId('');
            await fetchLinks();
        } catch (err) {
            setError('Failed to add link');
            console.error(err);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <Container>
            <Typography variant="h4" gutterBottom>
                Link Management
            </Typography>

            <Paper sx={{padding: 2, marginBottom: 3, display: 'flex', alignItems: 'center', gap: 2}}>
                <Typography variant="h6">
                    Add New Link (Hacker News Item ID)
                </Typography>
                <Box component="form" onSubmit={handleAddLink} sx={{display: 'flex', gap: 2}}>
                    <TextField
                        label="Item ID"
                        variant="outlined"
                        size="small"
                        value={newLinkId}
                        onChange={(e) => setNewLinkId(e.target.value)}
                        disabled={submitting}
                        type="number"
                    />
                    <Button
                        variant="contained"
                        type="submit"
                        disabled={submitting || !newLinkId}
                    >
                        {submitting ? <CircularProgress size={24}/> : 'Add'}
                    </Button>
                </Box>
            </Paper>

            {error && <Alert severity="error" sx={{mb: 2}}>{error}</Alert>}

            <TableContainer component={Paper}>
                <Table>
                    <TableHead>
                        <TableRow>
                            <TableCell>ID</TableCell>
                            <TableCell>URL</TableCell>
                            <TableCell>Date Added</TableCell>
                            <TableCell>Status</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={4} align="center">
                                    <CircularProgress/>
                                </TableCell>
                            </TableRow>
                        ) : links.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} align="center">
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
                                    <TableCell>{link.status || 'Scraped'}</TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </TableContainer>
        </Container>
    );
}