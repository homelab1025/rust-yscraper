import * as React from 'react';
import { Button, TableCell, TableRow } from '@mui/material';
import { type CommentDto } from '../api-client';

interface CommentRowProps {
    comment: CommentDto;
}

export default function CommentRow({ comment }: CommentRowProps): React.JSX.Element {
    return (
        <TableRow>
            <TableCell>{comment.text}</TableCell>
            <TableCell>{comment.user}</TableCell>
            <TableCell>{comment.date}</TableCell>
            <TableCell>
                <Button variant="text" onClick={() => {}}>Pick</Button>
                <Button variant="text" onClick={() => {}}>Discard</Button>
            </TableCell>
        </TableRow>
    );
}
