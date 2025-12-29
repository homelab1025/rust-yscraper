import * as React from 'react';
import {useEffect, useState} from 'react';
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
    TableRow,
    Typography
} from '@mui/material';
import axios from 'axios';
import {AddLink} from "../components/AddLink.tsx";

interface LinkDto {
    id: number;
    url: string;
    date_added: string;
    status?: string; // Adding status as mentioned in requirements, although not yet in backend
}

export default function LinkManagementPage(): React.JSX.Element {
    const [links, setLinks] = useState<LinkDto[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

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