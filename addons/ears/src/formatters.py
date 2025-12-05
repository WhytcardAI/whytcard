"""
Output formatters for transcription results
"""

from datetime import datetime
from pathlib import Path
from typing import Optional
import json

from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import cm
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, HRFlowable

from .engine import TranscriptionResult
from .config import settings


class MarkdownFormatter:
    """Format transcription results as Markdown"""

    def format(self, result: TranscriptionResult, title: Optional[str] = None) -> str:
        """Format transcription result as Markdown"""
        title = title or f"Transcription Audio"

        lines = [
            f"# {title}",
            "",
            f"**Date:** {result.timestamp.strftime('%Y-%m-%d %H:%M')}",
            f"**Langue:** {result.language.upper()}",
            f"**Modele:** {result.model_used}",
        ]

        if result.source_file:
            lines.append(f"**Fichier source:** {Path(result.source_file).name}")

        if result.duration:
            minutes = int(result.duration // 60)
            seconds = int(result.duration % 60)
            lines.append(f"**Duree:** {minutes}:{seconds:02d}")

        lines.extend([
            "",
            "---",
            "",
        ])

        # Raw transcription
        if settings.output.include_raw:
            lines.extend([
                "## Transcription brute",
                "",
                result.raw_text,
                "",
            ])

        # Corrected transcription
        if settings.output.include_corrected and result.corrected_text != result.raw_text:
            lines.extend([
                "## Transcription corrigee",
                "",
                result.corrected_text,
                "",
            ])

        # Summary
        if settings.output.include_summary and result.summary:
            lines.extend([
                "## Resume",
                "",
                result.summary,
                "",
            ])

        # Timestamps if available
        if settings.output.include_timestamps and result.segments:
            lines.extend([
                "## Segments temporels",
                "",
                "| Debut | Fin | Texte |",
                "|-------|-----|-------|",
            ])
            for seg in result.segments:
                start = f"{seg['start']:.1f}s"
                end = f"{seg['end']:.1f}s"
                text = seg['text'].strip()[:50]
                if len(seg['text'].strip()) > 50:
                    text += "..."
                lines.append(f"| {start} | {end} | {text} |")
            lines.append("")

        lines.extend([
            "---",
            "",
            f"*Genere par WhytCard Transcript v0.1.0*"
        ])

        return "\n".join(lines)

    def save(self, result: TranscriptionResult, output_path: str | Path, title: Optional[str] = None):
        """Save transcription result to Markdown file"""
        content = self.format(result, title)
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(content, encoding="utf-8")
        return output_path


class JSONFormatter:
    """Format transcription results as JSON"""

    def format(self, result: TranscriptionResult) -> str:
        """Format transcription result as JSON"""
        data = {
            "raw_text": result.raw_text,
            "corrected_text": result.corrected_text,
            "summary": result.summary,
            "language": result.language,
            "duration": result.duration,
            "model_used": result.model_used,
            "timestamp": result.timestamp.isoformat(),
            "source_file": result.source_file,
        }

        if settings.output.include_timestamps and result.segments:
            data["segments"] = result.segments

        if settings.output.include_confidence and result.confidence:
            data["confidence"] = result.confidence

        return json.dumps(data, indent=2, ensure_ascii=False)

    def save(self, result: TranscriptionResult, output_path: str | Path):
        """Save transcription result to JSON file"""
        content = self.format(result)
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(content, encoding="utf-8")
        return output_path


class PDFFormatter:
    """Format transcription results as PDF"""

    def _create_styles(self):
        """Create PDF styles"""
        styles = getSampleStyleSheet()

        styles.add(ParagraphStyle(
            name='MainTitle',
            parent=styles['Heading1'],
            fontSize=20,
            spaceAfter=20,
            alignment=TA_CENTER,
            textColor=colors.HexColor('#1a1a2e'),
            fontName='Helvetica-Bold'
        ))

        styles.add(ParagraphStyle(
            name='SectionHeader',
            parent=styles['Heading2'],
            fontSize=14,
            spaceBefore=15,
            spaceAfter=10,
            textColor=colors.HexColor('#16213e'),
            fontName='Helvetica-Bold'
        ))

        styles['BodyText'].fontSize = 10
        styles['BodyText'].spaceAfter = 8
        styles['BodyText'].alignment = TA_JUSTIFY
        styles['BodyText'].leading = 14

        return styles

    def _create_table_style(self):
        """Create table style"""
        return TableStyle([
            ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#16213e')),
            ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
            ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, 0), 10),
            ('BOTTOMPADDING', (0, 0), (-1, 0), 10),
            ('TOPPADDING', (0, 0), (-1, 0), 10),
            ('BACKGROUND', (0, 1), (-1, -1), colors.HexColor('#f8f9fa')),
            ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#dee2e6')),
            ('TOPPADDING', (0, 1), (-1, -1), 6),
            ('BOTTOMPADDING', (0, 1), (-1, -1), 6),
            ('LEFTPADDING', (0, 0), (-1, -1), 8),
            ('RIGHTPADDING', (0, 0), (-1, -1), 8),
        ])

    def save(self, result: TranscriptionResult, output_path: str | Path, title: Optional[str] = None):
        """Save transcription result to PDF file"""
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        doc = SimpleDocTemplate(
            str(output_path),
            pagesize=A4,
            rightMargin=2*cm,
            leftMargin=2*cm,
            topMargin=2*cm,
            bottomMargin=2*cm
        )

        styles = self._create_styles()
        story = []

        # Title
        title = title or "Transcription Audio"
        story.append(Paragraph(title, styles['MainTitle']))
        story.append(Spacer(1, 0.5*cm))

        # Metadata table
        meta_data = [
            ['Information', 'Valeur'],
            ['Date', result.timestamp.strftime('%Y-%m-%d %H:%M')],
            ['Langue', result.language.upper()],
            ['Modele', result.model_used],
        ]

        if result.source_file:
            meta_data.append(['Fichier', Path(result.source_file).name])

        if result.duration:
            minutes = int(result.duration // 60)
            seconds = int(result.duration % 60)
            meta_data.append(['Duree', f'{minutes}:{seconds:02d}'])

        meta_table = Table(meta_data, colWidths=[5*cm, 10*cm])
        meta_table.setStyle(self._create_table_style())
        story.append(meta_table)

        story.append(Spacer(1, 1*cm))
        story.append(HRFlowable(width="100%", thickness=1, color=colors.HexColor('#dee2e6')))
        story.append(Spacer(1, 0.5*cm))

        # Raw transcription
        if settings.output.include_raw:
            story.append(Paragraph("Transcription brute", styles['SectionHeader']))
            story.append(Paragraph(result.raw_text, styles['BodyText']))
            story.append(Spacer(1, 0.5*cm))

        # Corrected transcription
        if settings.output.include_corrected and result.corrected_text != result.raw_text:
            story.append(Paragraph("Transcription corrigee", styles['SectionHeader']))
            story.append(Paragraph(result.corrected_text, styles['BodyText']))
            story.append(Spacer(1, 0.5*cm))

        # Summary
        if settings.output.include_summary and result.summary:
            story.append(Paragraph("Resume", styles['SectionHeader']))
            story.append(Paragraph(result.summary.replace('\n', '<br/>'), styles['BodyText']))

        # Footer
        story.append(Spacer(1, 1*cm))
        story.append(HRFlowable(width="100%", thickness=1, color=colors.HexColor('#dee2e6')))
        story.append(Paragraph(
            f"Genere par WhytCard Transcript v0.1.0 - {datetime.now().strftime('%Y-%m-%d %H:%M')}",
            ParagraphStyle(
                name='Footer',
                fontSize=8,
                textColor=colors.HexColor('#6c757d'),
                alignment=TA_CENTER
            )
        ))

        doc.build(story)
        return output_path


def get_formatter(format_type: str = None):
    """Get appropriate formatter based on format type"""
    format_type = format_type or settings.output.format

    formatters = {
        "markdown": MarkdownFormatter,
        "md": MarkdownFormatter,
        "json": JSONFormatter,
        "pdf": PDFFormatter,
    }

    formatter_class = formatters.get(format_type.lower(), MarkdownFormatter)
    return formatter_class()
