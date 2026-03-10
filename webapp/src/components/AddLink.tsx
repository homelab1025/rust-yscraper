import { Alert, Box, Button, CircularProgress, Paper, TextField, Typography } from "@mui/material";
import * as React from "react";
import { useState } from "react";
import { useServices } from "../contexts/ServicesContext";

interface AddLinkProps {
    onSuccess?: () => void;
}

export function AddLink({ onSuccess }: AddLinkProps) {
    const { linksApi } = useServices();
    const [newLinkId, setNewLinkId] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleAddLink = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newLinkId) return;

        try {
            setSubmitting(true);
            await linksApi.scrapeLink({ item_id: parseInt(newLinkId) });
            setNewLinkId('');
            if (onSuccess) {
                onSuccess();
            }
        } catch (err) {
            setError('Failed to add link');
            console.error(err);
        } finally {
            setSubmitting(false);
        }
    };


    return (<Paper sx={{ padding: 2, marginBottom: 3, display: "flex", alignItems: "center", gap: 2 }}>
        <Typography variant="h6">
            Add New Link (Hacker News Item ID)
        </Typography>
        <Box component="form" onSubmit={handleAddLink} sx={{ display: "flex", gap: 2 }}>
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
                {submitting ? <CircularProgress size={24} /> : "Add"}
            </Button>
        </Box>

        {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}
    </Paper>);
}